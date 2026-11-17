# Phase 0.1 Bootstrap Verification Script (`scripts/verify-bootstrap.ps1`)
# Validates repository structure, governance contracts, documentation, agents, skills, rules, and workflows.

$ErrorActionPreference = "Stop"
$failures = 0
$checks = 0

function Assert-Condition {
    param(
        [bool]$Condition,
        [string]$Description
    )
    $script:checks++
    if ($Condition) {
        Write-Output "[PASS] $Description"
    } else {
        Write-Output "[FAIL] $Description"
        $script:failures++
    }
}

Write-Output "=== Running Stage 0 Phase 0.1 Bootstrap Verification ==="

# 1. Verify Git repository and branch
$branch = (git branch --show-current).Trim()
Assert-Condition ($branch -eq "main") "Git repository initialized on 'main' branch (actual: $branch)"

# 2. Verify required monorepo directories
$requiredDirs = @(
    "apps/web",
    "services/api",
    "services/worker",
    "services/browser-worker",
    "crates/core",
    "crates/cdm",
    "crates/url-security",
    "crates/web-extractor",
    "crates/github-extractor",
    "crates/code-intelligence",
    "crates/normalizer",
    "crates/deduplication",
    "crates/tokenizer",
    "crates/context-optimizer",
    "crates/chunking",
    "crates/exporters",
    "crates/storage",
    "crates/job-engine",
    "crates/observability",
    "docs",
    "deploy",
    "scripts",
    ".agent/skills",
    ".agent/rules",
    ".agent/workflows",
    ".agent/agents",
    ".github/workflows"
)
foreach ($dir in $requiredDirs) {
    Assert-Condition (Test-Path -Path $dir -PathType Container) "Directory exists: $dir"
}

# 3. Verify root governance & tracking files
$rootFiles = @(
    "README.md",
    "LICENSE",
    ".gitignore",
    ".gitattributes",
    ".editorconfig",
    "GLOBAL_ENGINEERING_CONTRACT.md",
    "REQUIREMENTS.md",
    "DECISIONS.md",
    "AGENTS.md",
    "ROADMAP.md",
    "IMPLEMENTATION_LEDGER.md",
    "PROJECT_STATUS.md",
    "TODO.md",
    "CHANGELOG.md"
)
foreach ($file in $rootFiles) {
    $exists = Test-Path -Path $file -PathType Leaf
    $nonEmpty = $exists -and ((Get-Item $file -Force).Length -gt 100)
    Assert-Condition $nonEmpty "Root governance file exists and is non-empty: $file"
}

# 4. Verify docs/ files
$docFiles = @(
    "docs/REQUIREMENTS.md",
    "docs/ARCHITECTURE.md",
    "docs/SECURITY.md",
    "docs/DATA_MODEL.md",
    "docs/API.md",
    "docs/DEVELOPMENT.md",
    "docs/DEPLOYMENT.md",
    "docs/USER_GUIDE.md",
    "docs/TESTING.md",
    "docs/PERFORMANCE.md",
    "docs/LICENSING.md",
    "docs/ROADMAP.md"
)
foreach ($doc in $docFiles) {
    $exists = Test-Path -Path $doc -PathType Leaf
    $nonEmpty = $exists -and ((Get-Item $doc -Force).Length -gt 100)
    Assert-Condition $nonEmpty "Documentation file exists and is non-empty: $doc"
}

# 5. Verify 10 skills and their 7 mandatory sections + YAML frontmatter
$skills = @(
    "architecture",
    "rust-engineering",
    "frontend-engineering",
    "web-extraction",
    "github-integration",
    "context-optimization",
    "security-engineering",
    "testing-quality",
    "devops-cicd",
    "documentation"
)
$requiredSkillSections = @(
    "Purpose",
    "When to Use",
    "Required Procedures",
    "Engineering Rules",
    "Validation Requirements",
    "Expected Outputs",
    "Common Failure Modes"
)
foreach ($skill in $skills) {
    $skillPath = ".agent/skills/$skill/SKILL.md"
    $exists = Test-Path -Path $skillPath -PathType Leaf
    Assert-Condition $exists "Skill file exists: $skillPath"
    if ($exists) {
        $content = Get-Content -Path $skillPath -Raw
        Assert-Condition ($content -match "(?s)^---\r?\nname:\s*$skill\r?\ndescription:") "Skill '$skill' has valid YAML frontmatter"
        foreach ($sec in $requiredSkillSections) {
            Assert-Condition ($content -match "(?i)##\s*\d*\.?\s*$sec") "Skill '$skill' contains mandatory section: $sec"
        }
    }
}

# 6. Verify 10 rules (and <24KB cap)
$rules = @(
    "00-global-engineering.md",
    "01-architecture.md",
    "02-rust.md",
    "03-frontend.md",
    "04-extraction-accuracy.md",
    "05-github-security.md",
    "06-testing.md",
    "07-documentation.md",
    "08-deployment.md",
    "09-no-mock-no-placeholder.md"
)
foreach ($rule in $rules) {
    $rulePath = ".agent/rules/$rule"
    $exists = Test-Path -Path $rulePath -PathType Leaf
    $sizeOk = $exists -and ((Get-Item $rulePath -Force).Length -gt 100) -and ((Get-Item $rulePath -Force).Length -lt 24000)
    Assert-Condition $sizeOk "Rule exists and satisfies size bounds (<24KB): $rulePath"
}

# 7. Verify 10 workflows
$workflows = @(
    "bootstrap-project.md",
    "execute-next-subphase.md",
    "verify-subphase.md",
    "complete-phase.md",
    "complete-stage.md",
    "security-audit.md",
    "performance-audit.md",
    "regression-audit.md",
    "release-certification.md",
    "project-status-review.md"
)
foreach ($wf in $workflows) {
    $wfPath = ".agent/workflows/$wf"
    $exists = Test-Path -Path $wfPath -PathType Leaf
    $nonEmpty = $exists -and ((Get-Item $wfPath -Force).Length -gt 100)
    Assert-Condition $nonEmpty "Workflow exists and is non-empty: $wfPath"
}

# 8. Verify 9 agent role definitions
$agents = @(
    "architecture-lead.md",
    "frontend-engineer.md",
    "rust-backend-engineer.md",
    "extraction-engineer.md",
    "github-integration-engineer.md",
    "security-reviewer.md",
    "qa-engineer.md",
    "devops-engineer.md",
    "documentation-reviewer.md"
)
foreach ($ag in $agents) {
    $agPath = ".agent/agents/$ag"
    $exists = Test-Path -Path $agPath -PathType Leaf
    Assert-Condition $exists "Agent role definition exists: $agPath"
}

# 9. Verify GitHub Actions workflows and YAML syntax
$ghWorkflows = @(
    "ci.yml",
    "build.yml",
    "container.yml",
    "release.yml",
    "security.yml",
    "dependency-review.yml",
    "codeql.yml"
)
foreach ($gw in $ghWorkflows) {
    $gwPath = ".github/workflows/$gw"
    $exists = Test-Path -Path $gwPath -PathType Leaf
    Assert-Condition $exists "GitHub Actions workflow exists: $gwPath"
}
$null = python scripts/validate-workflows.py 2>&1
Assert-Condition ($LASTEXITCODE -eq 0) "All GitHub Actions workflow YAML files valid via PyYAML parser"

# 10. Verify lint, formatting, security, and quality runner configs
$lintConfigs = @(
    "rustfmt.toml",
    ".prettierrc.json",
    ".prettierignore",
    "deny.toml",
    "scripts/check-all.ps1",
    "scripts/check-all.sh"
)
foreach ($lc in $lintConfigs) {
    $lcExists = Test-Path -Path $lc -PathType Leaf
    $lcNonEmpty = $lcExists -and ((Get-Item $lc -Force).Length -gt 10)
    Assert-Condition $lcNonEmpty "Lint/format/security configuration file exists and is non-empty: $lc"
}

# 11. Verify Podman Compose and Containerfile deployment configs
$deployConfigs = @(
    "deploy/podman-compose.yml",
    "deploy/Containerfile.api",
    "deploy/Containerfile.worker",
    "deploy/Containerfile.browser-worker",
    "deploy/Containerfile.web",
    "deploy/.env.example"
)
foreach ($dc in $deployConfigs) {
    $dcExists = Test-Path -Path $dc -PathType Leaf
    $dcNonEmpty = $dcExists -and ((Get-Item $dc -Force).Length -gt 10)
    Assert-Condition $dcNonEmpty "Deployment configuration file exists and is non-empty: $dc"
}
$null = python -c "import yaml; yaml.safe_load(open('deploy/podman-compose.yml'))" 2>&1
Assert-Condition ($LASTEXITCODE -eq 0) "deploy/podman-compose.yml is valid YAML"

# 12. Verify resource management and 30 GB ceiling scripts
$resourceScripts = @(
    "scripts/check-resources.ps1",
    "scripts/check-resources.sh",
    "scripts/clean-caches.ps1",
    "scripts/clean-caches.sh"
)
foreach ($rs in $resourceScripts) {
    $rsExists = Test-Path -Path $rs -PathType Leaf
    $rsNonEmpty = $rsExists -and ((Get-Item $rs -Force).Length -gt 10)
    Assert-Condition $rsNonEmpty "Resource management script exists and is non-empty: $rs"
}
$null = powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1 -Quiet 2>&1
Assert-Condition ($LASTEXITCODE -eq 0) "scripts/check-resources.ps1 executes successfully and satisfies 30 GB ceiling"

Write-Output "=== Verification Summary: $($checks - $failures)/$checks checks passed, $failures failures ==="
if ($failures -gt 0) {
    exit 1
}
exit 0
