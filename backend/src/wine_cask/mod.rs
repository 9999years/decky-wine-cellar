use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io::Write;
use crate::PeerMap;
use crate::wine_cask::wine_cask::{TaskType, WineCask};

pub mod flavors;
pub mod install;
pub mod uninstall;
pub mod wine_cask;
pub mod r#virtual;

pub fn generate_compatibility_tool_vdf(path: PathBuf, internal_name: &str, display_name: &str) {
    let mut file = File::create(path).expect("Failed to create file");
    writeln!(
        file,
        r#""compatibilitytools"
            {{
              "compat_tools"
              {{
                "{}"
                {{
                  "install_path" "."
                  "display_name" "{}"
                  "from_oslist"  "windows"
                  "to_oslist"    "linux"
                }}
              }}
            }}"#,
        internal_name, display_name
    ).expect("Failed to write to file");
}

fn copy_dir(source: &Path, destination: &Path) -> io::Result<()> {
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry_path.file_name().unwrap();
        let destination_path = destination.join(file_name);

        if entry_path.is_dir() {
            copy_dir(&entry_path, &destination_path)?;
        } else {
            fs::copy(&entry_path, &destination_path)?;
        }
    }

    Ok(())
}

fn recursive_delete_dir_entry(entry_path: &Path) -> std::io::Result<()> {
    if entry_path.is_dir() {
        for entry in fs::read_dir(entry_path)? {
            let entry = entry?;
            let path = entry.path();
            recursive_delete_dir_entry(&path)?;
        }
        fs::remove_dir(entry_path)?;
    } else {
        fs::remove_file(entry_path)?;
    }

    Ok(())
}

pub async fn process_queue(wine_cask: Arc<WineCask>, peer_map: PeerMap) {
    loop {
        match wine_cask.task_queue_pop_front().await {
            Some(task) => {
                if task.r#type == TaskType::InstallCompatibilityTool {
                    wine_cask
                        .install_compatibility_tool(task.install.unwrap(), &peer_map)
                        .await;
                }
            }
            None => {}
        }
    }
}