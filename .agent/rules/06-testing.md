# Rule 06: Testing & Verification Standards

1. **Coverage & Pass Targets:** Maintain >= 95% meaningful test coverage on critical core crates and a 100% pass rate on all mandatory CI checks.
2. **No Disabled Tests:** Never comment out, `#[ignore]`, or skip failing tests to make a build pass.
3. **Real & Adversarial Fixtures:** Test against real HTML/documentation fixtures, real/controlled GitHub repositories, SSRF/redirect bypass payloads, malformed inputs, and prompt-injection patterns.
4. **Golden Output Verification:** Validate CDM and exported Markdown/JSON against deterministic golden snapshots and source content hashes.
5. **Evidence Recording:** Every completed subphase must record exact test commands and outputs in `IMPLEMENTATION_LEDGER.md`.
