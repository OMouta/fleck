use crate::layer;
use crate::model::{
    Area, Asset, AssetSource, BlendMode, ClippingBehavior, ExportBackground, ExportParticipation,
    ImageAssetMetadata, ImageFormat, ImageObject, Layer, ObjectId, Padding, Point, RasterPixels,
    Rect, Size, Transform, TrimBehavior, Workspace,
};
use crate::persistence::{EmbeddedAssetBlob, WorkspacePackage};
use image::GenericImageView;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedImage {
    pub metadata: ImageAssetMetadata,
    pub rgba_pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImagePlacement {
    pub object_id: ObjectId,
    pub name: String,
    pub position: Point,
    pub scale: Size,
    pub rotation_degrees: f32,
    pub opacity: f32,
    pub crop_bounds: Option<Rect>,
    pub export_inclusion: ExportParticipation,
}

impl ImagePlacement {
    pub fn new(object_id: ObjectId, name: String, metadata: &ImageAssetMetadata) -> Self {
        Self {
            object_id,
            name,
            position: Point::ZERO,
            scale: Size {
                width: metadata.width as f32,
                height: metadata.height as f32,
            },
            rotation_degrees: 0.0,
            opacity: 1.0,
            crop_bounds: None,
            export_inclusion: ExportParticipation::Included,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedImage {
    pub asset: Asset,
    pub object: ImageObject,
    pub decoded: DecodedImage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddedImageImport {
    pub asset_id: ObjectId,
    pub name: String,
    pub bytes: Vec<u8>,
    pub placement: ImagePlacement,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkedImageImport {
    pub asset_id: ObjectId,
    pub name: String,
    pub path: PathBuf,
    pub placement: ImagePlacement,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageImportError {
    #[error("asset `{id}` was not found")]
    AssetNotFound { id: ObjectId },
    #[error("image object `{id}` was not found")]
    ObjectNotFound { id: ObjectId },
    #[error("asset `{id}` already exists")]
    DuplicateAssetId { id: ObjectId },
    #[error("image object `{id}` already exists")]
    DuplicateObjectId { id: ObjectId },
    #[error("image object opacity must be between 0.0 and 1.0")]
    InvalidOpacity,
    #[error("image scale must be positive")]
    InvalidScale,
    #[error("image crop bounds must be positive")]
    InvalidCropBounds,
    #[error("failed to decode image")]
    Decode(#[from] image::ImageError),
    #[error("failed to read image source")]
    Io(#[from] std::io::Error),
    #[error("embedded image bytes are not available to the workspace-only rasterizer")]
    EmbeddedBytesUnavailable,
    #[error("layer operation failed")]
    Layer(#[from] layer::LayerError),
    #[error("image object `{object_id}` references missing asset `{asset_id}`")]
    MissingObjectAsset {
        object_id: ObjectId,
        asset_id: ObjectId,
    },
}

pub type ImageImportResult<T> = Result<T, ImageImportError>;

pub fn decode_image_bytes(bytes: &[u8]) -> ImageImportResult<DecodedImage> {
    let format = image::guess_format(bytes).ok().map(image_format_from_crate);
    let image = image::load_from_memory(bytes)?;
    let (width, height) = image.dimensions();
    let color_type = image.color();
    let metadata = ImageAssetMetadata {
        width,
        height,
        format,
        color_type: format!("{color_type:?}"),
        has_alpha: color_type.has_alpha(),
    };

    Ok(DecodedImage {
        metadata,
        rgba_pixels: image.to_rgba8().into_raw(),
    })
}

pub fn import_embedded_image(
    package: &mut WorkspacePackage,
    request: EmbeddedImageImport,
) -> ImageImportResult<ImportedImage> {
    let decoded = decode_image_bytes(&request.bytes)?;
    ensure_asset_id_available(&package.workspace, &request.asset_id)?;
    let digest = digest_for_bytes(&request.bytes);
    let asset = Asset {
        id: request.asset_id.clone(),
        name: request.name,
        source: AssetSource::Embedded {
            digest: Some(digest.clone()),
        },
        media_type: decoded
            .metadata
            .format
            .and_then(media_type_for_format)
            .map(str::to_owned),
        color_profile: None,
        image_metadata: Some(decoded.metadata.clone()),
    };
    let object = place_image_asset(&mut package.workspace, asset.clone(), request.placement)?;
    package.embedded_assets.push(EmbeddedAssetBlob {
        asset_id: asset.id.clone(),
        digest: Some(digest),
        bytes: request.bytes,
    });

    Ok(ImportedImage {
        asset,
        object,
        decoded,
    })
}

pub fn import_linked_image(
    workspace: &mut Workspace,
    request: LinkedImageImport,
) -> ImageImportResult<ImportedImage> {
    let bytes = fs::read(&request.path)?;
    let decoded = decode_image_bytes(&bytes)?;
    ensure_asset_id_available(workspace, &request.asset_id)?;
    let asset = Asset {
        id: request.asset_id,
        name: request.name,
        source: AssetSource::Linked {
            path: request.path.to_string_lossy().into_owned(),
        },
        media_type: decoded
            .metadata
            .format
            .and_then(media_type_for_format)
            .map(str::to_owned),
        color_profile: None,
        image_metadata: Some(decoded.metadata.clone()),
    };
    let object = place_image_asset(workspace, asset.clone(), request.placement)?;

    Ok(ImportedImage {
        asset,
        object,
        decoded,
    })
}

pub fn place_image_asset(
    workspace: &mut Workspace,
    asset: Asset,
    placement: ImagePlacement,
) -> ImageImportResult<ImageObject> {
    ensure_asset_id_available(workspace, &asset.id)?;
    validate_placement(&placement)?;
    let object = image_object_from_placement(asset.id.clone(), placement)?;
    ensure_object_id_available(workspace, &object.id)?;
    workspace.assets.push(asset);
    workspace.image_objects.push(object.clone());
    Ok(object)
}

pub fn place_existing_asset(
    workspace: &mut Workspace,
    asset_id: ObjectId,
    placement: ImagePlacement,
) -> ImageImportResult<ImageObject> {
    require_asset(workspace, &asset_id)?;
    validate_placement(&placement)?;
    let object = image_object_from_placement(asset_id, placement)?;
    ensure_object_id_available(workspace, &object.id)?;
    workspace.image_objects.push(object.clone());
    Ok(object)
}

pub fn duplicate_image_object(
    workspace: &mut Workspace,
    object_id: &ObjectId,
    new_object_id: ObjectId,
) -> ImageImportResult<ImageObject> {
    ensure_object_id_available(workspace, &new_object_id)?;
    let object = require_object(workspace, object_id)?;
    let mut duplicate = object.clone();
    duplicate.id = new_object_id;
    duplicate.name = format!("{} Copy", duplicate.name);
    workspace.image_objects.push(duplicate.clone());
    Ok(duplicate)
}

pub fn replace_image_source(
    workspace: &mut Workspace,
    object_id: &ObjectId,
    new_asset_id: ObjectId,
) -> ImageImportResult<()> {
    require_asset(workspace, &new_asset_id)?;
    let object = require_object_mut(workspace, object_id)?;
    object.source_asset_id = new_asset_id;
    object.rasterized_layer_id = None;
    Ok(())
}

pub fn rasterize_image_object(
    workspace: &mut Workspace,
    object_id: &ObjectId,
    layer_id: ObjectId,
) -> ImageImportResult<()> {
    let object = require_object(workspace, object_id)?.clone();
    let asset = require_asset(workspace, &object.source_asset_id)?;
    let decoded = decoded_pixels_for_asset(asset)?;
    let object_rect = image_object_rect(&object);
    ensure_raster_target_areas(workspace, object_rect, &object.name);

    let intersections = workspace
        .areas
        .iter()
        .enumerate()
        .filter_map(|(index, area)| rect_intersection(area.bounds, object_rect).map(|rect| (index, rect)))
        .collect::<Vec<_>>();
    let mut first_layer_id = None;
    for (ordinal, (area_index, intersection)) in intersections.into_iter().enumerate() {
        let id = if ordinal == 0 {
            layer_id.clone()
        } else {
            ObjectId::new(format!("{}-{}", layer_id.as_str(), ordinal + 1))
                .expect("generated layer id should be valid")
        };
        let raster = crop_object_pixels(&decoded, &object, intersection);
        let layer = raster_layer_from_pixels(id.clone(), &object.name, intersection, raster);
        workspace.areas[area_index].layers.push(layer);
        first_layer_id.get_or_insert(id);
    }
    let image_object = require_object_mut(workspace, object_id)?;
    image_object.rasterized_layer_id = first_layer_id;
    Ok(())
}

fn ensure_raster_target_areas(workspace: &mut Workspace, rect: Rect, name: &str) {
    if workspace
        .areas
        .iter()
        .any(|area| rects_intersect(area.bounds, rect))
    {
        return;
    }
    let id = ObjectId::new(format!("area-{}", workspace.areas.len() + 1))
        .expect("generated area id should be valid");
    workspace.areas.push(Area {
        id: id.clone(),
        name: format!("{name} Area"),
        bounds: rect,
        layers: Vec::new(),
        padding: Padding::default(),
        background: ExportBackground::Transparent,
        trim: TrimBehavior::None,
        output_ids: Vec::new(),
        included_layer_ids: Vec::new(),
        excluded_layer_ids: Vec::new(),
        tags: Vec::new(),
        preset_id: None,
    });
}

fn image_object_rect(object: &ImageObject) -> Rect {
    Rect {
        x: object.position.x,
        y: object.position.y,
        width: object.scale.width,
        height: object.scale.height,
    }
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn rect_intersection(a: Rect, b: Rect) -> Option<Rect> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > left && bottom > top).then_some(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

fn decoded_pixels_for_asset(asset: &Asset) -> ImageImportResult<DecodedImage> {
    match &asset.source {
        AssetSource::Linked { path } => decode_image_bytes(&fs::read(path)?),
        AssetSource::Embedded { .. } => Err(ImageImportError::EmbeddedBytesUnavailable),
    }
}

fn crop_object_pixels(decoded: &DecodedImage, object: &ImageObject, intersection: Rect) -> RasterPixels {
    let width = intersection.width.ceil().max(1.0) as u32;
    let height = intersection.height.ceil().max(1.0) as u32;
    let mut pixels = vec![0; width as usize * height as usize * 4];
    let source_width = decoded.metadata.width.max(1);
    let source_height = decoded.metadata.height.max(1);
    let scale_x = source_width as f32 / object.scale.width.max(1.0);
    let scale_y = source_height as f32 / object.scale.height.max(1.0);

    for y in 0..height {
        for x in 0..width {
            let workspace_x = intersection.x + x as f32;
            let workspace_y = intersection.y + y as f32;
            let source_x = ((workspace_x - object.position.x) * scale_x)
                .floor()
                .clamp(0.0, (source_width - 1) as f32) as u32;
            let source_y = ((workspace_y - object.position.y) * scale_y)
                .floor()
                .clamp(0.0, (source_height - 1) as f32) as u32;
            let src = ((source_y * source_width + source_x) * 4) as usize;
            let dst = ((y * width + x) * 4) as usize;
            pixels[dst..dst + 4].copy_from_slice(&decoded.rgba_pixels[src..src + 4]);
        }
    }

    RasterPixels {
        width,
        height,
        pixels,
    }
}

fn raster_layer_from_pixels(
    id: ObjectId,
    name: &str,
    bounds: Rect,
    raster: RasterPixels,
) -> Layer {
    Layer {
        id,
        name: name.to_owned(),
        visible: true,
        opacity: 1.0,
        locked: false,
        position: Point {
            x: bounds.x,
            y: bounds.y,
        },
        bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: bounds.width.max(1.0),
            height: bounds.height.max(1.0),
        },
        blend_mode: BlendMode::Normal,
        alpha_channel: true,
        transform: Transform::default(),
        clipping: ClippingBehavior::None,
        mask_layer_id: None,
        group_id: None,
        export_participation: ExportParticipation::Included,
        raster: Some(raster),
    }
}

/// Decode `bytes` and register them as an embedded asset on `package`, without
/// placing an image object. The asset id and name are caller-supplied; returns
/// the decoded metadata so the caller can confirm dimensions/format.
pub fn register_embedded_asset(
    package: &mut WorkspacePackage,
    asset_id: ObjectId,
    name: String,
    bytes: Vec<u8>,
) -> ImageImportResult<(Asset, DecodedImage)> {
    let decoded = decode_image_bytes(&bytes)?;
    ensure_asset_id_available(&package.workspace, &asset_id)?;
    let digest = digest_for_bytes(&bytes);
    let asset = Asset {
        id: asset_id.clone(),
        name,
        source: AssetSource::Embedded {
            digest: Some(digest.clone()),
        },
        media_type: decoded
            .metadata
            .format
            .and_then(media_type_for_format)
            .map(str::to_owned),
        color_profile: None,
        image_metadata: Some(decoded.metadata.clone()),
    };
    package.workspace.assets.push(asset.clone());
    package.embedded_assets.push(EmbeddedAssetBlob {
        asset_id,
        digest: Some(digest),
        bytes,
    });
    Ok((asset, decoded))
}

pub fn reveal_asset_path(
    workspace: &Workspace,
    asset_id: &ObjectId,
) -> ImageImportResult<Option<PathBuf>> {
    let asset = require_asset(workspace, asset_id)?;
    Ok(match &asset.source {
        AssetSource::Linked { path } => Some(PathBuf::from(path)),
        AssetSource::Embedded { .. } => None,
    })
}

pub fn collect_linked_assets(workspace: &Workspace) -> Vec<&Asset> {
    workspace
        .assets
        .iter()
        .filter(|asset| matches!(asset.source, AssetSource::Linked { .. }))
        .collect()
}

fn image_object_from_placement(
    source_asset_id: ObjectId,
    placement: ImagePlacement,
) -> ImageImportResult<ImageObject> {
    Ok(ImageObject {
        id: placement.object_id,
        name: placement.name,
        source_asset_id,
        position: placement.position,
        scale: placement.scale,
        rotation_degrees: placement.rotation_degrees,
        opacity: placement.opacity,
        crop_bounds: placement.crop_bounds,
        rasterized_layer_id: None,
        export_inclusion: placement.export_inclusion,
    })
}

fn validate_placement(placement: &ImagePlacement) -> ImageImportResult<()> {
    if !(0.0..=1.0).contains(&placement.opacity) {
        return Err(ImageImportError::InvalidOpacity);
    }
    if placement.scale.width <= 0.0 || placement.scale.height <= 0.0 {
        return Err(ImageImportError::InvalidScale);
    }
    if placement
        .crop_bounds
        .is_some_and(|bounds| bounds.width <= 0.0 || bounds.height <= 0.0)
    {
        return Err(ImageImportError::InvalidCropBounds);
    }
    Ok(())
}

fn ensure_asset_id_available(workspace: &Workspace, id: &ObjectId) -> ImageImportResult<()> {
    if workspace.assets.iter().any(|asset| asset.id == *id) {
        Err(ImageImportError::DuplicateAssetId { id: id.clone() })
    } else {
        Ok(())
    }
}

fn ensure_object_id_available(workspace: &Workspace, id: &ObjectId) -> ImageImportResult<()> {
    if workspace
        .image_objects
        .iter()
        .any(|object| object.id == *id)
    {
        Err(ImageImportError::DuplicateObjectId { id: id.clone() })
    } else {
        Ok(())
    }
}

fn require_asset<'a>(workspace: &'a Workspace, id: &ObjectId) -> ImageImportResult<&'a Asset> {
    workspace
        .assets
        .iter()
        .find(|asset| asset.id == *id)
        .ok_or_else(|| ImageImportError::AssetNotFound { id: id.clone() })
}

fn require_object<'a>(
    workspace: &'a Workspace,
    id: &ObjectId,
) -> ImageImportResult<&'a ImageObject> {
    workspace
        .image_objects
        .iter()
        .find(|object| object.id == *id)
        .ok_or_else(|| ImageImportError::ObjectNotFound { id: id.clone() })
}

fn require_object_mut<'a>(
    workspace: &'a mut Workspace,
    id: &ObjectId,
) -> ImageImportResult<&'a mut ImageObject> {
    workspace
        .image_objects
        .iter_mut()
        .find(|object| object.id == *id)
        .ok_or_else(|| ImageImportError::ObjectNotFound { id: id.clone() })
}

fn image_format_from_crate(format: image::ImageFormat) -> ImageFormat {
    match format {
        image::ImageFormat::Png => ImageFormat::Png,
        image::ImageFormat::Jpeg => ImageFormat::Jpeg,
        image::ImageFormat::Gif => ImageFormat::Gif,
        image::ImageFormat::WebP => ImageFormat::WebP,
        image::ImageFormat::Bmp => ImageFormat::Bmp,
        image::ImageFormat::Tiff => ImageFormat::Tiff,
        image::ImageFormat::Ico => ImageFormat::Ico,
        _ => ImageFormat::Unknown,
    }
}

fn media_type_for_format(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Png => Some("image/png"),
        ImageFormat::Jpeg => Some("image/jpeg"),
        ImageFormat::Gif => Some("image/gif"),
        ImageFormat::WebP => Some("image/webp"),
        ImageFormat::Bmp => Some("image/bmp"),
        ImageFormat::Tiff => Some("image/tiff"),
        ImageFormat::Ico => Some("image/x-icon"),
        ImageFormat::Unknown => None,
    }
}

fn digest_for_bytes(bytes: &[u8]) -> String {
    let checksum = bytes.iter().fold(0_u64, |accumulator, byte| {
        accumulator.wrapping_add(*byte as u64)
    });
    format!("sum64:{:016x}:{}", checksum, bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Workspace;
    use image::{ImageBuffer, ImageFormat as CrateImageFormat, Rgba};
    use std::io::Cursor;

    #[test]
    fn decodes_image_bytes_with_metadata_and_pixels() {
        let bytes = png_bytes();
        let decoded = decode_image_bytes(&bytes).expect("decode");

        assert_eq!(decoded.metadata.width, 2);
        assert_eq!(decoded.metadata.height, 1);
        assert_eq!(decoded.metadata.format, Some(ImageFormat::Png));
        assert_eq!(decoded.rgba_pixels.len(), 8);
        assert!(decoded.metadata.has_alpha);
    }

    #[test]
    fn imports_embedded_image_into_package_with_blob() {
        let mut package = WorkspacePackage::new(workspace());
        let request = EmbeddedImageImport {
            asset_id: id("asset"),
            name: "paste.png".to_owned(),
            bytes: png_bytes(),
            placement: placement("object", "Pasted"),
        };

        let imported = import_embedded_image(&mut package, request).expect("import");

        assert_eq!(imported.asset.media_type.as_deref(), Some("image/png"));
        assert_eq!(package.workspace.assets.len(), 1);
        assert_eq!(package.workspace.image_objects.len(), 1);
        assert_eq!(package.embedded_assets.len(), 1);
        package.validate().expect("package validates");
    }

    #[test]
    fn imports_linked_image_with_source_path() {
        let path = temp_png_path();
        let mut workspace = workspace();
        let request = LinkedImageImport {
            asset_id: id("asset"),
            name: "linked.png".to_owned(),
            path: path.clone(),
            placement: placement("object", "Linked"),
        };

        import_linked_image(&mut workspace, request).expect("import");

        assert!(matches!(
            workspace.assets[0].source,
            AssetSource::Linked { .. }
        ));
        assert_eq!(
            reveal_asset_path(&workspace, &id("asset")).expect("path"),
            Some(path)
        );
    }

    #[test]
    fn duplicate_replace_and_rasterize_preserve_object_settings() {
        let mut workspace = workspace_with_asset("asset-a");
        let path = temp_png_path();
        workspace.assets.push(Asset {
            id: id("asset-b"),
            name: "replacement.png".to_owned(),
            source: AssetSource::Linked {
                path: path.to_string_lossy().into_owned(),
            },
            media_type: Some("image/png".to_owned()),
            color_profile: None,
            image_metadata: Some(metadata()),
        });
        place_existing_asset(&mut workspace, id("asset-a"), placement("object", "Placed"))
            .expect("place");
        workspace.image_objects[0].position = Point { x: 9.0, y: 10.0 };
        workspace.image_objects[0].rotation_degrees = 15.0;

        duplicate_image_object(&mut workspace, &id("object"), id("copy")).expect("duplicate");
        replace_image_source(&mut workspace, &id("object"), id("asset-b")).expect("replace");
        rasterize_image_object(&mut workspace, &id("object"), id("layer")).expect("rasterize");

        let object = workspace
            .image_objects
            .iter()
            .find(|object| object.id == id("object"))
            .expect("object");
        assert_eq!(object.source_asset_id, id("asset-b"));
        assert_eq!(object.position, Point { x: 9.0, y: 10.0 });
        assert_eq!(object.rotation_degrees, 15.0);
        assert_eq!(object.rasterized_layer_id, Some(id("layer")));
        assert_eq!(workspace.image_objects.len(), 2);
        assert_eq!(workspace.layers().count(), 1);
        let raster = workspace.areas[0].layers[0].raster.as_ref().expect("raster");
        assert_eq!(&raster.pixels[0..4], &[255, 0, 0, 255]);
        let _ = fs::remove_file(path);
    }

    fn workspace() -> Workspace {
        Workspace::empty(id("workspace"))
    }

    fn workspace_with_asset(asset_id: &str) -> Workspace {
        let mut workspace = workspace();
        workspace.assets.push(Asset {
            id: id(asset_id),
            name: "source.png".to_owned(),
            source: AssetSource::Embedded { digest: None },
            media_type: Some("image/png".to_owned()),
            color_profile: None,
            image_metadata: Some(metadata()),
        });
        workspace
    }

    fn placement(object_id: &str, name: &str) -> ImagePlacement {
        ImagePlacement::new(id(object_id), name.to_owned(), &metadata())
    }

    fn metadata() -> ImageAssetMetadata {
        ImageAssetMetadata {
            width: 2,
            height: 1,
            format: Some(ImageFormat::Png),
            color_type: "Rgba8".to_owned(),
            has_alpha: true,
        }
    }

    fn png_bytes() -> Vec<u8> {
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_fn(2, 1, |x, _| {
            if x == 0 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 128])
            }
        });
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, CrateImageFormat::Png)
            .expect("encode png");
        bytes.into_inner()
    }

    fn temp_png_path() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fleck-import-{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::write(&path, png_bytes()).expect("write png");
        path
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }
}
