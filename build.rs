use std::process::Command;

fn main() {
    let git_describe = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty=-dirty"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    // 获取 git commit hash
    let git_commit_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    // 获取 git tag（最新的 tag）
    let git_tag = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    // 获取构建时间
    let build_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 设置环境变量
    if let Some(describe) = git_describe {
        println!("cargo:rustc-env=GIT_DESCRIBE={}", describe);
    }

    if let Some(commit_hash) = git_commit_hash {
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash);
    }

    if let Some(tag) = git_tag {
        println!("cargo:rustc-env=GIT_TAG={}", tag);
    }

    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // 重新运行构建脚本如果 git 相关文件发生变化
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
