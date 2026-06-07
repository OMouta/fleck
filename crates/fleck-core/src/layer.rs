use crate::model::{
    BlendMode, ClippingBehavior, ExportParticipation, Layer, ObjectGroup, ObjectId, Point, Rect,
    Transform, Workspace,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NewLayer {
    pub id: ObjectId,
    pub name: String,
    pub bounds: Rect,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LayerError {
    #[error("layer `{id}` was not found")]
    NotFound { id: ObjectId },
    #[error("layer `{id}` is locked")]
    Locked { id: ObjectId },
    #[error("layer `{id}` already exists")]
    DuplicateId { id: ObjectId },
    #[error("layer index {index} is out of range")]
    IndexOutOfRange { index: usize },
    #[error("layer bounds must be positive")]
    NonPositiveBounds,
    #[error("no visible layers are available to flatten")]
    NoVisibleLayers,
}

pub type LayerResult<T> = Result<T, LayerError>;

pub fn create_layer(workspace: &mut Workspace, new_layer: NewLayer) -> LayerResult<()> {
    ensure_unique_layer_id(workspace, &new_layer.id)?;
    if new_layer.bounds.width <= 0.0 || new_layer.bounds.height <= 0.0 {
        return Err(LayerError::NonPositiveBounds);
    }

    workspace.layers.push(Layer {
        id: new_layer.id,
        name: new_layer.name,
        visible: true,
        opacity: 1.0,
        locked: false,
        position: new_layer.position,
        bounds: new_layer.bounds,
        blend_mode: BlendMode::Normal,
        alpha_channel: true,
        transform: Transform::default(),
        clipping: ClippingBehavior::None,
        mask_layer_id: None,
        group_id: None,
        export_participation: ExportParticipation::Included,
    });
    Ok(())
}

pub fn delete_layer(workspace: &mut Workspace, id: &ObjectId) -> LayerResult<Layer> {
    let index = require_layer_index(workspace, id)?;
    ensure_unlocked(&workspace.layers[index])?;
    Ok(workspace.layers.remove(index))
}

pub fn duplicate_layer(
    workspace: &mut Workspace,
    id: &ObjectId,
    new_id: ObjectId,
) -> LayerResult<()> {
    ensure_unique_layer_id(workspace, &new_id)?;
    let index = require_layer_index(workspace, id)?;
    let mut duplicate = workspace.layers[index].clone();
    duplicate.id = new_id;
    duplicate.name = format!("{} Copy", duplicate.name);
    workspace.layers.insert(index + 1, duplicate);
    Ok(())
}

pub fn rename_layer(workspace: &mut Workspace, id: &ObjectId, name: String) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.name = name;
    Ok(())
}

pub fn reorder_layer(
    workspace: &mut Workspace,
    id: &ObjectId,
    new_index: usize,
) -> LayerResult<()> {
    let index = require_layer_index(workspace, id)?;
    if new_index >= workspace.layers.len() {
        return Err(LayerError::IndexOutOfRange { index: new_index });
    }
    ensure_unlocked(&workspace.layers[index])?;
    let layer = workspace.layers.remove(index);
    workspace.layers.insert(new_index, layer);
    Ok(())
}

pub fn set_layer_group(
    workspace: &mut Workspace,
    id: &ObjectId,
    group_id: Option<ObjectId>,
) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.group_id = group_id;
    Ok(())
}

pub fn create_group(
    workspace: &mut Workspace,
    group_id: ObjectId,
    name: String,
    member_ids: Vec<ObjectId>,
) -> LayerResult<()> {
    if workspace
        .object_groups
        .iter()
        .any(|group| group.id == group_id)
    {
        return Err(LayerError::DuplicateId { id: group_id });
    }
    for member_id in &member_ids {
        require_layer(workspace, member_id)?;
    }
    workspace.object_groups.push(ObjectGroup {
        id: group_id,
        name,
        member_ids,
    });
    Ok(())
}

pub fn set_visibility(workspace: &mut Workspace, id: &ObjectId, visible: bool) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    layer.visible = visible;
    Ok(())
}

pub fn set_locked(workspace: &mut Workspace, id: &ObjectId, locked: bool) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    layer.locked = locked;
    Ok(())
}

pub fn set_opacity(workspace: &mut Workspace, id: &ObjectId, opacity: f32) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.opacity = opacity.clamp(0.0, 1.0);
    Ok(())
}

pub fn set_blend_mode(
    workspace: &mut Workspace,
    id: &ObjectId,
    blend_mode: BlendMode,
) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.blend_mode = blend_mode;
    Ok(())
}

pub fn set_clipping(
    workspace: &mut Workspace,
    id: &ObjectId,
    clipping: ClippingBehavior,
) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.clipping = clipping;
    Ok(())
}

pub fn set_mask(
    workspace: &mut Workspace,
    id: &ObjectId,
    mask_layer_id: Option<ObjectId>,
) -> LayerResult<()> {
    if let Some(mask_id) = &mask_layer_id {
        require_layer(workspace, mask_id)?;
    }
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.mask_layer_id = mask_layer_id;
    Ok(())
}

pub fn rasterize_layer(workspace: &mut Workspace, id: &ObjectId) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    layer.transform = Transform::default();
    layer.alpha_channel = true;
    Ok(())
}

pub fn trim_to_visible_pixels(workspace: &mut Workspace, id: &ObjectId) -> LayerResult<()> {
    let layer = require_layer_mut(workspace, id)?;
    ensure_unlocked(layer)?;
    if layer.opacity <= 0.0 || !layer.visible {
        layer.bounds.width = 0.0;
        layer.bounds.height = 0.0;
    }
    Ok(())
}

pub fn merge_down(workspace: &mut Workspace, id: &ObjectId) -> LayerResult<()> {
    let source_index = require_layer_index(workspace, id)?;
    if source_index == 0 {
        return Err(LayerError::IndexOutOfRange {
            index: source_index,
        });
    }
    let target_index = source_index - 1;
    ensure_unlocked(&workspace.layers[source_index])?;
    ensure_unlocked(&workspace.layers[target_index])?;

    let source = workspace.layers.remove(source_index);
    let target = &mut workspace.layers[target_index];
    target.bounds = union_layer_bounds(target, &source);
    target.position = Point::ZERO;
    target.name = format!("Merged {}", target.name);
    target.opacity = 1.0;
    target.blend_mode = BlendMode::Normal;
    target.alpha_channel = target.alpha_channel || source.alpha_channel;
    Ok(())
}

pub fn flatten_visible_layers(
    workspace: &mut Workspace,
    flattened_id: ObjectId,
) -> LayerResult<()> {
    ensure_unique_layer_id(workspace, &flattened_id)?;
    let visible_layers = workspace
        .layers
        .iter()
        .filter(|layer| layer.visible && layer.opacity > 0.0)
        .cloned()
        .collect::<Vec<_>>();
    if visible_layers.is_empty() {
        return Err(LayerError::NoVisibleLayers);
    }
    for layer in &visible_layers {
        ensure_unlocked(layer)?;
    }

    let bounds = visible_layers
        .iter()
        .skip(1)
        .fold(layer_workspace_rect(&visible_layers[0]), |bounds, layer| {
            union_rect(bounds, layer_workspace_rect(layer))
        });
    workspace
        .layers
        .retain(|layer| !layer.visible || layer.opacity <= 0.0);
    workspace.layers.push(Layer {
        id: flattened_id,
        name: "Flattened Layers".to_owned(),
        visible: true,
        opacity: 1.0,
        locked: false,
        position: Point::ZERO,
        bounds,
        blend_mode: BlendMode::Normal,
        alpha_channel: true,
        transform: Transform::default(),
        clipping: ClippingBehavior::None,
        mask_layer_id: None,
        group_id: None,
        export_participation: ExportParticipation::Included,
    });
    Ok(())
}

fn require_layer<'a>(workspace: &'a Workspace, id: &ObjectId) -> LayerResult<&'a Layer> {
    workspace
        .layers
        .iter()
        .find(|layer| layer.id == *id)
        .ok_or_else(|| LayerError::NotFound { id: id.clone() })
}

fn require_layer_mut<'a>(
    workspace: &'a mut Workspace,
    id: &ObjectId,
) -> LayerResult<&'a mut Layer> {
    workspace
        .layers
        .iter_mut()
        .find(|layer| layer.id == *id)
        .ok_or_else(|| LayerError::NotFound { id: id.clone() })
}

fn require_layer_index(workspace: &Workspace, id: &ObjectId) -> LayerResult<usize> {
    workspace
        .layers
        .iter()
        .position(|layer| layer.id == *id)
        .ok_or_else(|| LayerError::NotFound { id: id.clone() })
}

fn ensure_unique_layer_id(workspace: &Workspace, id: &ObjectId) -> LayerResult<()> {
    if workspace.layers.iter().any(|layer| layer.id == *id) {
        Err(LayerError::DuplicateId { id: id.clone() })
    } else {
        Ok(())
    }
}

fn ensure_unlocked(layer: &Layer) -> LayerResult<()> {
    if layer.locked {
        Err(LayerError::Locked {
            id: layer.id.clone(),
        })
    } else {
        Ok(())
    }
}

fn union_layer_bounds(a: &Layer, b: &Layer) -> Rect {
    union_rect(layer_workspace_rect(a), layer_workspace_rect(b))
}

fn layer_workspace_rect(layer: &Layer) -> Rect {
    Rect {
        x: layer.position.x + layer.bounds.x,
        y: layer.position.y + layer.bounds.y,
        width: layer.bounds.width * layer.transform.scale_x,
        height: layer.bounds.height * layer.transform.scale_y,
    }
}

fn union_rect(a: Rect, b: Rect) -> Rect {
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = (a.x + a.width).max(b.x + b.width);
    let bottom = (a.y + a.height).max(b.y + b.height);
    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_delete_duplicate_and_reorder_layers() {
        let mut workspace = workspace();
        create_layer(&mut workspace, new_layer("base")).expect("create base");
        duplicate_layer(&mut workspace, &id("base"), id("copy")).expect("duplicate");
        reorder_layer(&mut workspace, &id("copy"), 0).expect("reorder");

        assert_eq!(workspace.layers[0].id, id("copy"));
        assert_eq!(workspace.layers[1].id, id("base"));

        let deleted = delete_layer(&mut workspace, &id("copy")).expect("delete");
        assert_eq!(deleted.id, id("copy"));
        assert_eq!(workspace.layers.len(), 1);
    }

    #[test]
    fn locked_layers_reject_mutating_operations() {
        let mut workspace = workspace();
        create_layer(&mut workspace, new_layer("base")).expect("create base");
        set_locked(&mut workspace, &id("base"), true).expect("lock");

        let error = set_opacity(&mut workspace, &id("base"), 0.5).expect_err("locked");
        assert!(matches!(error, LayerError::Locked { .. }));
        assert!(delete_layer(&mut workspace, &id("base")).is_err());
    }

    #[test]
    fn visibility_lock_opacity_blend_clipping_and_mask_are_set() {
        let mut workspace = workspace();
        create_layer(&mut workspace, new_layer("base")).expect("create base");
        create_layer(&mut workspace, new_layer("mask")).expect("create mask");

        set_visibility(&mut workspace, &id("base"), false).expect("hide");
        set_opacity(&mut workspace, &id("base"), 0.25).expect("opacity");
        set_blend_mode(&mut workspace, &id("base"), BlendMode::Multiply).expect("blend");
        set_clipping(&mut workspace, &id("base"), ClippingBehavior::ClipToGroup).expect("clip");
        set_mask(&mut workspace, &id("base"), Some(id("mask"))).expect("mask");

        let layer = require_layer(&workspace, &id("base")).expect("layer");
        assert!(!layer.visible);
        assert_eq!(layer.opacity, 0.25);
        assert_eq!(layer.blend_mode, BlendMode::Multiply);
        assert_eq!(layer.clipping, ClippingBehavior::ClipToGroup);
        assert_eq!(layer.mask_layer_id, Some(id("mask")));
    }

    #[test]
    fn merge_and_flatten_produce_single_composite_bounds() {
        let mut workspace = workspace();
        let mut left = new_layer("left");
        left.position = Point { x: 0.0, y: 0.0 };
        let mut right = new_layer("right");
        right.position = Point { x: 8.0, y: 0.0 };
        create_layer(&mut workspace, left).expect("left");
        create_layer(&mut workspace, right).expect("right");

        merge_down(&mut workspace, &id("right")).expect("merge down");
        assert_eq!(workspace.layers.len(), 1);
        assert_eq!(workspace.layers[0].bounds.width, 24.0);

        duplicate_layer(&mut workspace, &id("left"), id("copy")).expect("duplicate");
        flatten_visible_layers(&mut workspace, id("flat")).expect("flatten");

        assert_eq!(workspace.layers.len(), 1);
        assert_eq!(workspace.layers[0].id, id("flat"));
    }

    fn workspace() -> Workspace {
        Workspace::empty(id("workspace"))
    }

    fn new_layer(value: &str) -> NewLayer {
        NewLayer {
            id: id(value),
            name: value.to_owned(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 16.0,
            },
            position: Point::ZERO,
        }
    }

    fn id(value: &str) -> ObjectId {
        ObjectId::new(value).expect("test id should be valid")
    }
}
