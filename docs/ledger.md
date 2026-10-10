# Disclosure ledger

Most privacy problems are not missing policies. They are policies that no longer
match the code. The ledger lists what the code does, then compares it with what the
privacy policy says.

cargo run -p shipcheck-cli -- examples/drift-demo --ledger
cargo run -p shipcheck-cli -- examples/drift-demo --ledger --format json


A normal scan includes the same findings.

## What goes into the ledger

- **Third parties**: about 40 known services (analytics, advertising, monitoring, AI,
  payments, email, support, maps, fonts, captcha, backends, login), found by domain in
  code and by package name in `package.json` and `requirements.txt`.
- **Cookies**: code that writes cookies or uses a cookie or session library.
- **Personal data**: form fields for email, phone, postal address and date of birth,
  geolocation calls, and camera or microphone access.

## What counts as the policy

Files named `privacy`, `privacy-policy`, `privacy-notice`, `privacy-statement`,
`data-policy`, `cookie-policy` or `cookie-notice` with a document or page extension, and
`index`, `page` or `readme` files inside a folder with one of those names. Policy files
are never treated as evidence of what the code does.

## Findings

| Rule | Meaning | Severity |
|---|---|---|
| DRIFT-001 | The code uses a service the policy never names | High for advertising, medium for analytics, monitoring and AI, low otherwise |
| DRIFT-002 | The code sets cookies, the policy never mentions cookies | Medium |
| DRIFT-003 | The code collects personal data the policy never mentions | High for location and camera or microphone, medium for phone, address and date of birth, low for email |
| DRIFT-004 | The policy promises something the code contradicts | High |
| DRIFT-005 | The policy names a service no code uses | Info |

DRIFT-004 reads promises such as "we do not use cookies", "we never share your data with
third parties", "we do not track you" and "we do not collect any personal information".
A promise followed by a condition ("except", "unless", "from children") is not treated
as a flat promise.

Without a policy file nothing is compared, because the ordinary legal rule LEGAL-001
already reports the missing policy.

## Limits

- It is a set of heuristics, not legal advice. A clean result does not mean a policy is
  compliant, and a finding is a prompt to look, not a verdict.
- Names are matched as words in the policy text. A policy that describes a service
  without naming it will be reported.
- Only the files in the scanned folder are read. Code in other repositories is not seen,
  which is why DRIFT-005 is informational.
- Rust source files are not read yet.
