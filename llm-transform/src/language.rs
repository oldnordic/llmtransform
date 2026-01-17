use serde::{Deserialize, Serialize};
use std::path::Path;

/// Supported programming languages for file type detection
///
/// This enum represents languages that llm-transform can identify
/// and process. Each variant corresponds to a language with
/// specific file extensions and syntax handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Rust (.rs)
    Rust,
    /// C (.c, .h)
    C,
    /// C++ (.cpp, .cc, .cxx, .hpp, .hxx, .hh)
    Cpp,
    /// Java (.java)
    Java,
    /// JavaScript (.js, .mjs, .cjs)
    JavaScript,
    /// TypeScript (.ts, .tsx)
    TypeScript,
    /// Python (.py, .pyi)
    Python,
    /// Unknown or unsupported language
    Unknown,
}

impl Language {
    /// Get the file extensions associated with this language
    ///
    /// # Returns
    /// A slice of extension strings (without the dot)
    pub fn extensions(&self) -> &[&str] {
        match self {
            Language::Rust => &["rs"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx", "hh"],
            Language::Java => &["java"],
            Language::JavaScript => &["js", "mjs", "cjs"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Python => &["py", "pyi"],
            Language::Unknown => &[],
        }
    }

    /// Get the language name as a string
    pub fn name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Python => "Python",
            Language::Unknown => "Unknown",
        }
    }

    /// Check if this language is supported for editing
    pub fn is_supported(&self) -> bool {
        !matches!(self, Language::Unknown)
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Detect the programming language from a file path
///
/// This function examines the file extension and returns the corresponding
/// Language variant. If the extension is not recognized, it returns
/// Language::Unknown.
///
/// # Arguments
/// * `path` - A path-like object (file path or just filename)
///
/// # Returns
/// * The detected Language variant
///
/// # Examples
/// ```
/// use llm_transform::{Language, detect_language};
/// assert_eq!(detect_language("main.rs"), Language::Rust);
/// assert_eq!(detect_language("header.hpp"), Language::Cpp);
/// assert_eq!(detect_language("script.py"), Language::Python);
/// assert_eq!(detect_language("unknown.xyz"), Language::Unknown);
/// ```
pub fn detect_language<P: AsRef<Path>>(path: P) -> Language {
    let path_ref = path.as_ref();

    // Get the file extension (without the dot)
    let extension = path_ref
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    // Map extension to language
    match extension {
        "rs" => Language::Rust,
        "c" => Language::C,
        "h" => Language::C,
        "cpp" | "cc" | "cxx" => Language::Cpp,
        "hpp" | "hxx" | "hh" => Language::Cpp,
        "java" => Language::Java,
        "js" | "mjs" | "cjs" => Language::JavaScript,
        "ts" | "tsx" => Language::TypeScript,
        "py" | "pyi" => Language::Python,
        _ => Language::Unknown,
    }
}

/// Create an extension filter for the given language
///
/// Useful for glob patterns or file filtering operations.
///
/// # Arguments
/// * `language` - The language to get extensions for
///
/// # Returns
/// * A comma-separated string of extensions (without dots, suitable for glob patterns)
///
/// # Example
/// ```
/// use llm_transform::Language;
/// let rust_exts = Language::Rust.extension_filter();
/// assert_eq!(rust_exts, "rs");
/// ```
impl Language {
    pub fn extension_filter(&self) -> String {
        self.extensions().join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        assert_eq!(detect_language("main.rs"), Language::Rust);
        assert_eq!(detect_language("lib.rs"), Language::Rust);
        assert_eq!(detect_language("/path/to/module.rs"), Language::Rust);
    }

    #[test]
    fn test_detect_c() {
        assert_eq!(detect_language("main.c"), Language::C);
        assert_eq!(detect_language("header.h"), Language::C);
        assert_eq!(detect_language("/usr/include/stdio.h"), Language::C);
    }

    #[test]
    fn test_detect_cpp() {
        assert_eq!(detect_language("main.cpp"), Language::Cpp);
        assert_eq!(detect_language("impl.cc"), Language::Cpp);
        assert_eq!(detect_language("source.cxx"), Language::Cpp);
        assert_eq!(detect_language("header.hpp"), Language::Cpp);
        assert_eq!(detect_language("header.hxx"), Language::Cpp);
        assert_eq!(detect_language("header.hh"), Language::Cpp);
    }

    #[test]
    fn test_detect_java() {
        assert_eq!(detect_language("Main.java"), Language::Java);
        assert_eq!(detect_language("com/example/Test.java"), Language::Java);
    }

    #[test]
    fn test_detect_javascript() {
        assert_eq!(detect_language("app.js"), Language::JavaScript);
        assert_eq!(detect_language("module.mjs"), Language::JavaScript);
        assert_eq!(detect_language("script.cjs"), Language::JavaScript);
    }

    #[test]
    fn test_detect_typescript() {
        assert_eq!(detect_language("app.ts"), Language::TypeScript);
        assert_eq!(detect_language("component.tsx"), Language::TypeScript);
    }

    #[test]
    fn test_detect_python() {
        assert_eq!(detect_language("main.py"), Language::Python);
        assert_eq!(detect_language("type.pyi"), Language::Python);
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_language("unknown.xyz"), Language::Unknown);
        assert_eq!(detect_language("README"), Language::Unknown);
        assert_eq!(detect_language("no_extension"), Language::Unknown);
        assert_eq!(detect_language(""), Language::Unknown);
    }

    #[test]
    fn test_extensions() {
        assert_eq!(Language::Rust.extensions(), &["rs"]);
        assert_eq!(Language::C.extensions(), &["c", "h"]);
        assert_eq!(Language::Cpp.extensions(), &["cpp", "cc", "cxx", "hpp", "hxx", "hh"]);
        assert_eq!(Language::Java.extensions(), &["java"]);
        assert_eq!(Language::JavaScript.extensions(), &["js", "mjs", "cjs"]);
        assert_eq!(Language::TypeScript.extensions(), &["ts", "tsx"]);
        assert_eq!(Language::Python.extensions(), &["py", "pyi"]);
        assert_eq!(Language::Unknown.extensions(), &[] as &[&str]);
    }

    #[test]
    fn test_is_supported() {
        assert!(Language::Rust.is_supported());
        assert!(Language::C.is_supported());
        assert!(Language::Cpp.is_supported());
        assert!(Language::Java.is_supported());
        assert!(Language::JavaScript.is_supported());
        assert!(Language::TypeScript.is_supported());
        assert!(Language::Python.is_supported());
        assert!(!Language::Unknown.is_supported());
    }

    #[test]
    fn test_display() {
        assert_eq!(Language::Rust.to_string(), "Rust");
        assert_eq!(Language::Cpp.to_string(), "C++");
        assert_eq!(Language::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_extension_filter() {
        assert_eq!(Language::Rust.extension_filter(), "rs");
        assert_eq!(Language::Cpp.extension_filter(), "cpp,cc,cxx,hpp,hxx,hh");
        assert_eq!(Language::Python.extension_filter(), "py,pyi");
        assert_eq!(Language::Unknown.extension_filter(), "");
    }
}
