//! 版本信息模块，从构建时环境变量获取 git 信息

use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

/// 版本信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub tag: String,
    pub commit_hash: String,
    pub build_time: String,
    pub git_describe: String,
}

impl Version {
 
    pub fn new() -> Self {
        Self {
            tag: Self::get_tag(),
            commit_hash: Self::get_commit_hash(),
            build_time: Self::get_build_time(),
            git_describe: Self::get_git_describe(),
        }
    }

  
    fn get_commit_hash() -> String {
        option_env!("GIT_COMMIT_HASH")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn get_tag() -> String {
        option_env!("GIT_TAG")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

   
    fn get_build_time() -> String {
        option_env!("BUILD_TIME")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }


    fn get_git_describe() -> String {
        option_env!("GIT_DESCRIBE")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }


    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    
    pub fn to_string(&self) -> String {
        if self.tag == "unknown" {
            format!("commit:{} built:{}", self.commit_hash, self.build_time)
        } else {
            format!("{} (commit:{}) built:{}", self.tag, self.commit_hash, self.build_time)
        }
    }

    pub fn to_short_string(&self) -> String {
        if self.tag == "unknown" {
            format!("commit:{}", self.commit_hash)
        } else {
            format!("{}:{}", self.tag, self.commit_hash)
        }
    }

    #[allow(dead_code)]
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "tag": self.tag,
            "commit_hash": self.commit_hash,
            "build_time": self.build_time,
            "git_describe": self.git_describe,
            "full_version": self.to_string(),
            "short_version": self.to_short_string(),
        })
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref VERSION: Version = Version::new();
}

pub fn version() -> &'static Version {
    &VERSION
}


#[allow(dead_code)]
pub fn full_version() -> String {
    VERSION.to_string()
}


#[allow(dead_code)]
pub fn short_version() -> String {
    VERSION.to_short_string()
}
