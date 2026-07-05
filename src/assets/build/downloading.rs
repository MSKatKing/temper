use build_print::info;
use std::path::PathBuf;

const SERVER_JAR_URL: &str =
    "https://piston-data.mojang.com/v1/objects/6bce4ef400e4efaa63a13d5e6f6b500be969ef81/server.jar";

pub fn generate() {
    if let Ok((server_jar, assets_dir)) = setup() {
        if !server_jar {
            download_jar(assets_dir.join("server.jar"));
            let java_path = which::which("java").expect("Failed to find java in PATH");

            let _ = std::process::Command::new(java_path)
                .arg("-DbundlerMainClass=net.minecraft.data.Main")
                .arg("-jar")
                .arg("server.jar")
                .arg("--all")
                .current_dir(workspace_root::get_workspace_root_directory().expect("Failed to get workspace root directory").join("assets/generated"))
                .output()
                .expect("Failed to execute java command");
            info!("Finished generating assets");
        }
    } else {
        println!("cargo:error=Setup failed");
    }
    
    
}
fn download_jar(path: PathBuf) {
    info!(
        "Downloading server jar, this could take a sec. If this takes unusually long and network \
    usage is low, its a bug and should be reported"
    );
    let server_jar_bytes = reqwest::blocking::Client::new()
        .get(SERVER_JAR_URL)
        .send()
        .expect("Failed to send request")
        .error_for_status()
        .expect("Failed to download server jar")
        .bytes()
        .expect("Failed to read response bytes")
        .as_ref()
        .to_owned();
    info!("Downloaded server jar, writing to {:?}", path);
    std::fs::write(path, server_jar_bytes).expect("Failed to write server jar to disk");
    info!("Wrote server jar to disk");
}
fn setup() -> Result<(bool, PathBuf), ()> {
    let root = workspace_root::get_workspace_root_directory()
        .expect("Failed to get workspace root directory")
        .canonicalize()
        .expect("Failed to canonicalize root directory");
    let generated_assets_dir = root.join("assets/generated");
    if !generated_assets_dir.exists() {
        return if generated_assets_dir
            .parent()
            .expect("Failed to get parent directory")
            .exists()
        {
            std::fs::create_dir_all(&generated_assets_dir)
                .expect("Failed to create generated assets directory");
            Err(())
        } else {
            println!(
                "cargo:error=Parent directory of generated assets does not exist, is {:?} the right directory for the project?",
                root
            );
            Err(())
        };
    }

    if generated_assets_dir.join("server.jar").exists() {
        Ok((true, generated_assets_dir))
    } else {
        Ok((false, generated_assets_dir))
    }
}
