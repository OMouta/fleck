fn main() {
    println!("{} CLI scaffold", fleck_core::APP_NAME);
    for boundary in fleck_core::ownership_boundaries() {
        println!("{}: {}", boundary.owner, boundary.responsibility);
    }
}
