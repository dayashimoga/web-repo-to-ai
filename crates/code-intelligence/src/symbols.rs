//! # AST Symbol Extraction & Declaration Mapping
//!
//! Extracts functions, types, traits, interfaces, classes, and macros
//! across supported programming languages to determine structural code semantics.

use serde::{Deserialize, Serialize};

use crate::language::Language;

/// Kind of symbol declared in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// Function definition or standalone procedure.
    Function,
    /// Method associated with an object, struct, or class.
    Method,
    /// Data structure or record definition.
    Struct,
    /// Object-oriented class definition.
    Class,
    /// Interface specification contract.
    Interface,
    /// Trait or typeclass specification contract.
    Trait,
    /// Enumerated type or variant collection.
    Enum,
    /// Type alias or typedef.
    TypeAlias,
    /// Compile-time constant or static variable.
    Constant,
    /// Syntactic macro or metaprogramming template.
    Macro,
    /// Submodule or namespace.
    Module,
    /// Module import or dependency inclusion.
    Import,
}

impl SymbolKind {
    /// Returns the snake_case name of the symbol kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Struct => "struct",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Trait => "trait",
            Self::Enum => "enum",
            Self::TypeAlias => "type_alias",
            Self::Constant => "constant",
            Self::Macro => "macro",
            Self::Module => "module",
            Self::Import => "import",
        }
    }

    /// Returns true if this symbol represents a type or interface definition.
    #[must_use]
    pub const fn is_type_definition(self) -> bool {
        matches!(
            self,
            Self::Struct
                | Self::Class
                | Self::Interface
                | Self::Trait
                | Self::Enum
                | Self::TypeAlias
        )
    }

    /// Returns true if this symbol represents an executable function or method.
    #[must_use]
    pub const fn is_callable(self) -> bool {
        matches!(self, Self::Function | Self::Method)
    }
}

/// Declared visibility or export scope of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Publicly exported / accessible outside module or package.
    Public,
    /// Accessible only to subtypes or package hierarchy.
    Protected,
    /// Private to module, file, or enclosing scope.
    Private,
    /// Internal / crate-visible / package-private.
    Internal,
}

/// Extracted symbol declaration with location and signature metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDeclaration {
    /// Declared identifier name.
    pub name: String,
    /// Symbol categorization.
    pub kind: SymbolKind,
    /// Declared visibility scope.
    pub visibility: Visibility,
    /// 1-based line number of declaration in source file.
    pub line_number: usize,
    /// Full declaration line or signature header.
    pub signature: String,
    /// Associated doc comment if present immediately preceding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_comment: Option<String>,
}

/// Multi-language symbol extractor.
pub struct SymbolExtractor;

impl SymbolExtractor {
    /// Extracts all symbol declarations from code content for a given language.
    #[must_use]
    pub fn extract(language: &Language, content: &str) -> Vec<SymbolDeclaration> {
        match language {
            Language::Rust => Self::extract_rust(content),
            Language::TypeScript | Language::JavaScript => {
                Self::extract_typescript_javascript(content)
            }
            Language::Python => Self::extract_python(content),
            Language::Go => Self::extract_go(content),
            Language::Java | Language::CSharp => Self::extract_java_csharp(content),
            Language::Cpp | Language::C => Self::extract_c_cpp(content),
            _ => Self::extract_generic(content),
        }
    }

    fn extract_rust(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();
        let mut pending_doc: Option<String> = None;

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                let doc_line = trimmed.trim_start_matches('/').trim();
                if let Some(existing) = pending_doc.as_mut() {
                    existing.push(' ');
                    existing.push_str(doc_line);
                } else {
                    pending_doc = Some(doc_line.to_string());
                }
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with("//") {
                pending_doc = None;
                continue;
            }

            let (visibility, rest) =
                if trimmed.starts_with("pub(crate) ") || trimmed.starts_with("pub(super) ") {
                    (
                        Visibility::Internal,
                        trimmed.split_once(' ').map(|x| x.1).unwrap_or(trimmed),
                    )
                } else if let Some(stripped) = trimmed.strip_prefix("pub ") {
                    (Visibility::Public, stripped)
                } else {
                    (Visibility::Private, trimmed)
                };

            let rest_trimmed = rest.trim_start();

            // Function
            if rest_trimmed.starts_with("fn ")
                || rest_trimmed.starts_with("async fn ")
                || rest_trimmed.starts_with("const fn ")
            {
                if let Some(name) = extract_identifier_after(rest_trimmed, "fn ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Function,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("struct ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "struct ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Struct,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("enum ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "enum ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Enum,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("trait ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "trait ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Trait,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("type ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "type ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::TypeAlias,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("const ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "const ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Constant,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("macro_rules! ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "macro_rules! ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Macro,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("mod ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "mod ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Module,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("use ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "use ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Import,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else {
                pending_doc = None;
            }
        }

        symbols
    }

    fn extract_typescript_javascript(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();
        let mut pending_doc: Option<String> = None;

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("/**") || trimmed.starts_with('*') {
                let doc_line = trimmed
                    .trim_start_matches('/')
                    .trim_start_matches('*')
                    .trim();
                if !doc_line.is_empty() {
                    if let Some(existing) = pending_doc.as_mut() {
                        existing.push(' ');
                        existing.push_str(doc_line);
                    } else {
                        pending_doc = Some(doc_line.to_string());
                    }
                }
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with("//") {
                pending_doc = None;
                continue;
            }

            let (visibility, rest) = if let Some(stripped) = trimmed.strip_prefix("export default ")
            {
                (Visibility::Public, stripped)
            } else if let Some(stripped) = trimmed.strip_prefix("export ") {
                (Visibility::Public, stripped)
            } else {
                (Visibility::Private, trimmed)
            };

            let rest_trimmed = rest.trim_start();

            if rest_trimmed.starts_with("function ") || rest_trimmed.starts_with("async function ")
            {
                if let Some(name) = extract_identifier_after(rest_trimmed, "function ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Function,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("class ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "class ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Class,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("interface ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "interface ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Interface,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("type ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "type ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::TypeAlias,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("enum ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "enum ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Enum,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("const ") && rest_trimmed.contains(" = (") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "const ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Function,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("const ") {
                if let Some(name) = extract_identifier_after(rest_trimmed, "const ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Constant,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: pending_doc.take(),
                    });
                }
            } else if rest_trimmed.starts_with("import ") {
                symbols.push(SymbolDeclaration {
                    name: "import".to_string(),
                    kind: SymbolKind::Import,
                    visibility: Visibility::Private,
                    line_number: line_num,
                    signature: trimmed.to_string(),
                    doc_comment: None,
                });
            } else {
                pending_doc = None;
            }
        }

        symbols
    }

    fn extract_python(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                let is_method = line.starts_with("    ") || line.starts_with('\t');
                let target = if trimmed.starts_with("async def ") {
                    "async def "
                } else {
                    "def "
                };
                if let Some(name) = extract_identifier_after(trimmed, target) {
                    let visibility = if name.starts_with('_') && !name.starts_with("__") {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: if is_method {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        },
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = extract_identifier_after(trimmed, "class ") {
                    let visibility = if name.starts_with('_') {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Class,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                symbols.push(SymbolDeclaration {
                    name: "import".to_string(),
                    kind: SymbolKind::Import,
                    visibility: Visibility::Private,
                    line_number: line_num,
                    signature: trimmed.to_string(),
                    doc_comment: None,
                });
            }
        }

        symbols
    }

    fn extract_go(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if let Some(stripped) = trimmed.strip_prefix("func ") {
                let rest = stripped.trim_start();
                let (kind, after_recv) = if rest.starts_with('(') {
                    (
                        SymbolKind::Method,
                        rest.split_once(')')
                            .map(|x| x.1.trim_start())
                            .unwrap_or(rest),
                    )
                } else {
                    (SymbolKind::Function, rest)
                };

                let name: String = after_recv
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();

                if !name.is_empty() {
                    let visibility = if name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };
                    symbols.push(SymbolDeclaration {
                        name,
                        kind,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if let Some(stripped) = trimmed.strip_prefix("type ") {
                let after_type = stripped.trim_start();
                let name: String = after_type
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();

                if !name.is_empty() {
                    let kind = if after_type.contains("struct") {
                        SymbolKind::Struct
                    } else if after_type.contains("interface") {
                        SymbolKind::Interface
                    } else {
                        SymbolKind::TypeAlias
                    };

                    let visibility = if name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    symbols.push(SymbolDeclaration {
                        name,
                        kind,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.starts_with("import ") {
                symbols.push(SymbolDeclaration {
                    name: "import".to_string(),
                    kind: SymbolKind::Import,
                    visibility: Visibility::Private,
                    line_number: line_num,
                    signature: trimmed.to_string(),
                    doc_comment: None,
                });
            }
        }

        symbols
    }

    fn extract_java_csharp(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            let visibility = if trimmed.starts_with("public ") {
                Visibility::Public
            } else if trimmed.starts_with("protected ") {
                Visibility::Protected
            } else if trimmed.starts_with("private ") {
                Visibility::Private
            } else {
                Visibility::Internal
            };

            if trimmed.contains(" class ") {
                if let Some(name) = extract_identifier_after(trimmed, "class ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Class,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.contains(" interface ") {
                if let Some(name) = extract_identifier_after(trimmed, "interface ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Interface,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.contains(" enum ") {
                if let Some(name) = extract_identifier_after(trimmed, "enum ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Enum,
                        visibility,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            }
        }

        symbols
    }

    fn extract_c_cpp(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("struct ") {
                if let Some(name) = extract_identifier_after(trimmed, "struct ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Struct,
                        visibility: Visibility::Public,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = extract_identifier_after(trimmed, "class ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Class,
                        visibility: Visibility::Public,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.starts_with("enum ") {
                if let Some(name) = extract_identifier_after(trimmed, "enum ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Enum,
                        visibility: Visibility::Public,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            }
        }

        symbols
    }

    fn extract_generic(content: &str) -> Vec<SymbolDeclaration> {
        let mut symbols = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("function ") {
                if let Some(name) = extract_identifier_after(trimmed, "function ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Function,
                        visibility: Visibility::Public,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            } else if trimmed.starts_with("class ") {
                if let Some(name) = extract_identifier_after(trimmed, "class ") {
                    symbols.push(SymbolDeclaration {
                        name,
                        kind: SymbolKind::Class,
                        visibility: Visibility::Public,
                        line_number: line_num,
                        signature: trimmed.to_string(),
                        doc_comment: None,
                    });
                }
            }
        }

        symbols
    }
}

fn extract_identifier_after(haystack: &str, prefix: &str) -> Option<String> {
    if let Some(idx) = haystack.find(prefix) {
        let after = &haystack[idx + prefix.len()..];
        let ident: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !ident.is_empty() {
            return Some(ident);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_declarations() {
        let rust_code = "\
/// Documentation for QueueProcessor
pub struct QueueProcessor {
    capacity: usize,
}

pub trait ProcessorEngine {
    fn execute(&self);
}

pub async fn run_worker(id: u64) -> Result<(), Error> {
    Ok(())
}

fn internal_helper() {}
";
        let symbols = SymbolExtractor::extract(&Language::Rust, rust_code);
        assert_eq!(symbols.len(), 5);

        assert_eq!(symbols[0].name, "QueueProcessor");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
        assert_eq!(symbols[0].visibility, Visibility::Public);
        assert!(symbols[0]
            .doc_comment
            .as_ref()
            .unwrap()
            .contains("Documentation for QueueProcessor"));

        assert_eq!(symbols[1].name, "ProcessorEngine");
        assert_eq!(symbols[1].kind, SymbolKind::Trait);

        assert_eq!(symbols[2].name, "execute");
        assert_eq!(symbols[2].kind, SymbolKind::Function);

        assert_eq!(symbols[3].name, "run_worker");
        assert_eq!(symbols[3].kind, SymbolKind::Function);
        assert_eq!(symbols[3].visibility, Visibility::Public);

        assert_eq!(symbols[4].name, "internal_helper");
        assert_eq!(symbols[4].visibility, Visibility::Private);
    }

    #[test]
    fn test_extract_typescript_declarations() {
        let ts_code = "\
export interface ClientConfig {
    apiKey: string;
}

export class ApiService {
    fetch() {}
}

export function initialize(config: ClientConfig) {}
";
        let symbols = SymbolExtractor::extract(&Language::TypeScript, ts_code);
        assert_eq!(symbols.len(), 3);

        assert_eq!(symbols[0].name, "ClientConfig");
        assert_eq!(symbols[0].kind, SymbolKind::Interface);

        assert_eq!(symbols[1].name, "ApiService");
        assert_eq!(symbols[1].kind, SymbolKind::Class);

        assert_eq!(symbols[2].name, "initialize");
        assert_eq!(symbols[2].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_python_declarations() {
        let py_code = "\
class Orchestrator:
    def __init__(self):
        pass

    def run_cycle(self):
        pass

def standalone_task():
    pass
";
        let symbols = SymbolExtractor::extract(&Language::Python, py_code);
        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].name, "Orchestrator");
        assert_eq!(symbols[0].kind, SymbolKind::Class);
        assert_eq!(symbols[1].name, "__init__");
        assert_eq!(symbols[1].kind, SymbolKind::Method);
        assert_eq!(symbols[2].name, "run_cycle");
        assert_eq!(symbols[2].kind, SymbolKind::Method);
        assert_eq!(symbols[3].name, "standalone_task");
        assert_eq!(symbols[3].kind, SymbolKind::Function);
    }
}
