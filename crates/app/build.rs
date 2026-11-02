fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo::rerun-if-changed=../../assets/icons/resources.rc");
        println!("cargo::rerun-if-changed=../../assets/icons/app.ico");
        let _ = embed_resource::compile("../../assets/icons/resources.rc", embed_resource::NONE);
    }
}
