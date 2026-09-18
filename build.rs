fn main() {
    // `sqlx::migrate!` embeds these files; rebuild when a migration is added or edited.
    println!("cargo:rerun-if-changed=migrations");
}
