//! Sources, sanitizers and sinks, expressed as data.

use shipcheck_core::Severity;

/// A class of vulnerability that taint analysis can report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Class {
    Sqli,
    CommandInjection,
    CodeInjection,
    Xss,
    Ssrf,
    PathTraversal,
    Deserialization,
    OpenRedirect,
    TemplateInjection,
}

/// Rule metadata for one class.
pub(crate) struct Info {
    pub id: &'static str,
    pub severity: Severity,
    pub message: &'static str,
    pub fix: &'static str,
}

impl Class {
    pub(crate) const fn info(self) -> Info {
        match self {
            Self::Sqli => Info {
                id: "SEC-101",
                severity: Severity::High,
                message: "User input reaches a SQL query (CWE-89).",
                fix: "Use parameterized queries and bind user values as parameters.",
            },
            Self::CommandInjection => Info {
                id: "SEC-102",
                severity: Severity::Critical,
                message: "User input reaches a shell command (CWE-78).",
                fix: "Avoid the shell: pass an argument list and validate input against an allowlist.",
            },
            Self::CodeInjection => Info {
                id: "SEC-103",
                severity: Severity::Critical,
                message: "User input reaches code evaluation (CWE-94).",
                fix: "Never evaluate user input. Parse it as data instead.",
            },
            Self::Xss => Info {
                id: "SEC-104",
                severity: Severity::High,
                message: "User input reaches HTML output without escaping (CWE-79).",
                fix: "Escape the value, use textContent, or sanitize the HTML first.",
            },
            Self::Ssrf => Info {
                id: "SEC-105",
                severity: Severity::High,
                message: "User input controls an outgoing request URL (CWE-918).",
                fix: "Validate the URL against an allowlist of hosts and block private address ranges.",
            },
            Self::PathTraversal => Info {
                id: "SEC-106",
                severity: Severity::High,
                message: "User input reaches a file path (CWE-22).",
                fix: "Resolve the path against a fixed base directory and reject anything that escapes it.",
            },
            Self::Deserialization => Info {
                id: "SEC-107",
                severity: Severity::High,
                message: "User input reaches an unsafe deserializer (CWE-502).",
                fix: "Use a safe format such as JSON, or a safe loader.",
            },
            Self::OpenRedirect => Info {
                id: "SEC-108",
                severity: Severity::Medium,
                message: "User input controls a redirect target (CWE-601).",
                fix: "Redirect only to relative paths or an allowlist of known destinations.",
            },
            Self::TemplateInjection => Info {
                id: "SEC-109",
                severity: Severity::Critical,
                message: "User input reaches a template engine as code (CWE-1336).",
                fix: "Pass user values as template variables, never as the template itself.",
            },
        }
    }
}

/// A group of call names that are dangerous when given user input.
pub(crate) struct CallSink {
    pub class: Class,
    pub names: &'static [&'static str],
    /// Match the whole callee text only, not just a trailing method name.
    pub exact: bool,
    /// Check every argument instead of only the first.
    pub any_arg: bool,
}

pub(crate) const JS_SOURCES: &[&str] = &[
    "req.query",
    "req.body",
    "req.params",
    "req.headers",
    "req.cookies",
    "req.url",
    "req.originalUrl",
    "request.query",
    "request.body",
    "request.params",
    "request.headers",
    "ctx.query",
    "ctx.params",
    "ctx.request.body",
    "location.search",
    "location.hash",
    "location.href",
    "document.location",
    "window.location",
    "document.cookie",
    "document.referrer",
    "process.argv",
];

pub(crate) const PY_SOURCES: &[&str] = &[
    "request.args",
    "request.form",
    "request.values",
    "request.json",
    "request.data",
    "request.cookies",
    "request.headers",
    "request.GET",
    "request.POST",
    "request.COOKIES",
    "request.META",
    "sys.argv",
];

pub(crate) const JS_SANITIZERS: &[&str] = &[
    "parseInt",
    "parseFloat",
    "Number",
    "Boolean",
    "encodeURIComponent",
    "encodeURI",
    "escape",
    "escapeHtml",
    "sanitize",
    "sanitizeHtml",
    "basename",
];

pub(crate) const PY_SANITIZERS: &[&str] = &[
    "int",
    "float",
    "bool",
    "escape",
    "quote",
    "quote_plus",
    "secure_filename",
    "basename",
    "sanitize",
    "clean",
];

pub(crate) const JS_REFLECT_CALLS: &[&str] = &[
    "res.send",
    "res.write",
    "res.end",
    "response.send",
    "response.write",
    "response.end",
];

pub(crate) const PY_SHELL_CALLS: &[&str] = &[
    "subprocess.run",
    "subprocess.call",
    "subprocess.check_call",
    "subprocess.check_output",
    "subprocess.Popen",
];

pub(crate) const JS_SINKS: &[CallSink] = &[
    CallSink {
        class: Class::CodeInjection,
        names: &[
            "eval",
            "Function",
            "vm.runInNewContext",
            "vm.runInThisContext",
            "vm.runInContext",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::CommandInjection,
        names: &[
            "exec",
            "execSync",
            "child_process.exec",
            "child_process.execSync",
            "cp.exec",
            "cp.execSync",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::Sqli,
        names: &[
            "query",
            "execute",
            "raw",
            "$queryRawUnsafe",
            "$executeRawUnsafe",
        ],
        exact: false,
        any_arg: false,
    },
    CallSink {
        class: Class::Ssrf,
        names: &[
            "fetch",
            "axios",
            "axios.get",
            "axios.post",
            "axios.put",
            "axios.patch",
            "axios.delete",
            "axios.request",
            "http.get",
            "https.get",
            "http.request",
            "https.request",
            "got",
            "got.get",
            "got.post",
            "request",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::PathTraversal,
        names: &[
            "fs.readFile",
            "fs.readFileSync",
            "fs.createReadStream",
            "fs.writeFile",
            "fs.writeFileSync",
            "fs.unlink",
            "fs.unlinkSync",
            "fs.readdir",
            "fs.promises.readFile",
            "readFile",
            "readFileSync",
            "res.sendFile",
            "res.download",
            "response.sendFile",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::OpenRedirect,
        names: &["res.redirect", "response.redirect"],
        exact: true,
        any_arg: true,
    },
    CallSink {
        class: Class::Xss,
        names: &["document.write", "document.writeln"],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::Xss,
        names: &["insertAdjacentHTML"],
        exact: false,
        any_arg: true,
    },
];

pub(crate) const PY_SINKS: &[CallSink] = &[
    CallSink {
        class: Class::CodeInjection,
        names: &["eval", "exec"],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::TemplateInjection,
        names: &["render_template_string", "Template", "jinja2.Template"],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::CommandInjection,
        names: &[
            "os.system",
            "os.popen",
            "subprocess.getoutput",
            "subprocess.getstatusoutput",
            "commands.getoutput",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::Sqli,
        names: &[
            "execute",
            "executemany",
            "executescript",
            "raw",
            "read_sql",
            "read_sql_query",
        ],
        exact: false,
        any_arg: false,
    },
    CallSink {
        class: Class::Ssrf,
        names: &[
            "requests.get",
            "requests.post",
            "requests.put",
            "requests.delete",
            "requests.head",
            "requests.patch",
            "requests.request",
            "urllib.request.urlopen",
            "urlopen",
            "httpx.get",
            "httpx.post",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::PathTraversal,
        names: &[
            "open",
            "os.remove",
            "os.unlink",
            "os.listdir",
            "shutil.rmtree",
            "send_file",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::PathTraversal,
        names: &["send_from_directory"],
        exact: true,
        any_arg: true,
    },
    CallSink {
        class: Class::Deserialization,
        names: &[
            "pickle.loads",
            "pickle.load",
            "yaml.load",
            "marshal.loads",
            "jsonpickle.decode",
        ],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::OpenRedirect,
        names: &["redirect", "flask.redirect"],
        exact: true,
        any_arg: false,
    },
    CallSink {
        class: Class::Xss,
        names: &["Markup", "mark_safe", "flask.Markup"],
        exact: true,
        any_arg: false,
    },
];
