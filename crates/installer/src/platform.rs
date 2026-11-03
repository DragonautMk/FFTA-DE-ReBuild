use std::{io,path::Path};

pub const PLAYER:&str=if cfg!(windows){"FFTARecomp.exe"}else{"FFTARecomp"};
pub const RECOMP:&str=if cfg!(windows){"recomp.exe"}else{"recomp"};

pub fn executable(path:&Path)->io::Result<()> {
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        let mut permission=std::fs::metadata(path)?.permissions();
        permission.set_mode(permission.mode()|0o111);std::fs::set_permissions(path,permission)?;
    }
    #[cfg(not(unix))] let _=path;
    Ok(())
}

fn exec_path(path:&Path)->String {
    path.to_string_lossy().replace('\\',"\\\\").replace('"',"\\\"")
        .replace('`',"\\`").replace('$',"\\$").replace('%',"%%")
}
pub fn desktop_entry(dir:&Path)->io::Result<String> {
    let text=dir.to_string_lossy();
    if text.contains(['\n','\r']) {return Err(io::Error::new(io::ErrorKind::InvalidInput,"install path contains a newline"));}
    Ok(format!("[Desktop Entry]\nType=Application\nName=FFTA DE-ReBuild\nExec=\"{}\"\nPath={}\nTerminal=false\nCategories=Game;\n",
        exec_path(&dir.join(PLAYER)),text.replace('\\',"\\\\")))
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn desktop_paths_do_not_expand_shell_or_field_codes() {
        let entry=desktop_entry(Path::new("/tmp/Game $x%f")).unwrap();
        assert!(entry.contains("Game \\$x%%f"));assert!(entry.contains("Terminal=false"));
        assert!(desktop_entry(Path::new("/tmp/Game\nExec=bad")).is_err());
    }
}
