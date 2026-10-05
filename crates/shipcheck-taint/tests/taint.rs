use std::path::Path;

use shipcheck_taint::{analyze, Lang};

fn found(lang: Lang, src: &str) -> Vec<(String, u32)> {
    let mut hits: Vec<(String, u32)> = analyze(lang, src, "test")
        .into_iter()
        .map(|finding| (finding.rule_id, finding.line))
        .collect();
    hits.sort();
    hits
}

fn hit(id: &str, line: u32) -> (String, u32) {
    (id.to_owned(), line)
}

const NONE: Vec<(String, u32)> = Vec::new();

#[test]
fn detects_language_from_extension() {
    assert_eq!(Lang::from_path(Path::new("a/b.ts")), Some(Lang::TypeScript));
    assert_eq!(Lang::from_path(Path::new("x.py")), Some(Lang::Python));
    assert_eq!(Lang::from_path(Path::new("README.md")), None);
}

#[test]
fn js_template_sql_from_params_is_flagged() {
    let src = r#"app.get("/user", (req, res) => {
  const id = req.params.id;
  db.query(`SELECT * FROM users WHERE id = ${id}`);
});
"#;
    assert_eq!(found(Lang::JavaScript, src), vec![hit("SEC-101", 3)]);
}

#[test]
fn js_parameterized_query_is_clean() {
    let src = r#"app.get("/user", (req, res) => {
  const id = req.params.id;
  db.query("SELECT * FROM users WHERE id = ?", [id]);
});
"#;
    assert_eq!(found(Lang::JavaScript, src), NONE);
}

#[test]
fn js_sanitizer_clears_taint() {
    let src = r#"const id = parseInt(req.params.id, 10);
db.query("SELECT * FROM users WHERE id = " + id);
"#;
    assert_eq!(found(Lang::JavaScript, src), NONE);
}

#[test]
fn js_command_injection_from_query() {
    let src = r#"app.get("/ping", (req, res) => {
  exec("ping -c 1 " + req.query.host);
});
"#;
    assert_eq!(found(Lang::JavaScript, src), vec![hit("SEC-102", 2)]);
}

#[test]
fn js_ssrf_from_query() {
    let src = r#"app.get("/proxy", async (req, res) => {
  const r = await fetch(req.query.url);
});
"#;
    assert_eq!(found(Lang::JavaScript, src), vec![hit("SEC-105", 2)]);
}

#[test]
fn js_destructured_input_reaches_file_path() {
    let src = r#"app.get("/file", (req, res) => {
  const { name } = req.query;
  fs.readFile("./uploads/" + name, () => {});
});
"#;
    assert_eq!(found(Lang::JavaScript, src), vec![hit("SEC-106", 3)]);
}

#[test]
fn js_basename_clears_path_taint() {
    let src = r#"const name = path.basename(req.query.name);
fs.readFile("./uploads/" + name, () => {});
"#;
    assert_eq!(found(Lang::JavaScript, src), NONE);
}

#[test]
fn js_inner_html_from_location_hash() {
    let src = r#"const q = location.hash;
document.getElementById("out").innerHTML = q;
"#;
    assert_eq!(found(Lang::JavaScript, src), vec![hit("SEC-104", 2)]);
}

#[test]
fn js_reflected_html_response() {
    let src = r#"app.get("/hi", (req, res) => {
  res.send(`<h1>Hello ${req.query.name}</h1>`);
});
"#;
    assert_eq!(found(Lang::JavaScript, src), vec![hit("SEC-104", 2)]);
}

#[test]
fn js_taint_does_not_leak_between_functions() {
    let src = r#"function a(req) {
  const id = req.query.id;
  return id;
}
function b() {
  const id = 1;
  db.query("SELECT * FROM t WHERE id = " + id);
}
"#;
    assert_eq!(found(Lang::JavaScript, src), NONE);
}

#[test]
fn typescript_is_supported() {
    let src = r#"app.get("/u", (req: Request, res: Response) => {
  const id: string = req.params.id;
  db.query("SELECT * FROM users WHERE id = " + id);
});
"#;
    assert_eq!(found(Lang::TypeScript, src), vec![hit("SEC-101", 3)]);
}

#[test]
fn python_os_system_with_request_data() {
    let src = r#"name = request.args.get("name")
os.system("echo " + name)
"#;
    assert_eq!(found(Lang::Python, src), vec![hit("SEC-102", 2)]);
}

#[test]
fn python_fstring_sql_is_flagged() {
    let src = r#"def lookup(cursor):
    uid = request.args["id"]
    cursor.execute(f"SELECT * FROM users WHERE id = {uid}")
"#;
    assert_eq!(found(Lang::Python, src), vec![hit("SEC-101", 3)]);
}

#[test]
fn python_parameterized_sql_is_clean() {
    let src = r#"def lookup(cursor):
    uid = request.args["id"]
    cursor.execute("SELECT * FROM users WHERE id = %s", (uid,))
"#;
    assert_eq!(found(Lang::Python, src), NONE);
}

#[test]
fn python_subprocess_needs_shell_true() {
    let risky = r#"host = request.args.get("host")
subprocess.run("ping " + host, shell=True)
"#;
    let safe = r#"host = request.args.get("host")
subprocess.run(["ping", host])
"#;
    assert_eq!(found(Lang::Python, risky), vec![hit("SEC-102", 2)]);
    assert_eq!(found(Lang::Python, safe), NONE);
}

#[test]
fn python_input_reaches_open() {
    let src = "path = input()\nopen(path)\n";
    assert_eq!(found(Lang::Python, src), vec![hit("SEC-106", 2)]);
}

#[test]
fn python_pickle_of_request_data() {
    let src = "data = request.data\nobj = pickle.loads(data)\n";
    assert_eq!(found(Lang::Python, src), vec![hit("SEC-107", 2)]);
}
