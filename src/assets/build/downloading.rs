
const SERVER_JAR_URL: &str = "https://piston-data.mojang.com/v1/objects/6bce4ef400e4efaa63a13d5e6f6b500be969ef81/server.jar";


pub fn generate() {
    if let Ok(server_jar) = setup() {
        if !server_jar {
            download_jar()
        }
    } else {
        println!("cargo:error=Setup failed");
    }
}
fn download_jar() {}
fn setup() -> Result<bool, ()>{
    let root = workspace_root::get_workspace_root_directory().expect("Failed to get workspace root directory");
    build_print::info!("cargo:warning=Workspace root directory: {:?}", root);
    let generated_assets_dir = root.join("assets/generated");
    if !generated_assets_dir.exists() {
        return if generated_assets_dir.parent().expect("Failed to get parent directory").exists() {
            std::fs::create_dir_all(&generated_assets_dir).expect("Failed to create generated assets directory");
            build_print::info!("cargo:warning=Created generated assets directory at {:?}", generated_assets_dir);
            Err(())
        } else {
            build_print::info!("cargo:error=Parent directory of generated assets does not exist, is {:?} the right directory for the project?", root);
            Err(())
        }
    } else {
        build_print::info!("cargo:warning=Generated assets directory already exists at {:?}", generated_assets_dir);
    }
    
    if generated_assets_dir.join("server.jar").exists() {
        build_print::info!("cargo:warning=Server jar already exists at {:?}", generated_assets_dir.join("server.jar"));
        Ok(true)
    } else {
        build_print::info!("cargo:warning=Server jar does not exist at {:?}", generated_assets_dir.join("server.jar"));
        Ok(false)
    }
}