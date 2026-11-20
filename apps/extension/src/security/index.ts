// Security Module Barrel Exports for web-repo-to-ai browser extension

export {
  EXTENSION_CSP,
  PRIVILEGED_SCHEMES,
  isPrivilegedUrl,
  isSafeHttpUrl,
  validateApiBaseUrl,
  validateTargetUrl,
  type UrlValidationResult,
} from "./policy";

export { maskApiKey, sanitizeError, sanitizeLog, isSensitiveHeader } from "./redaction";

export { escapeHtml, stripDangerousAttributes, sanitizeHtmlContent } from "./sanitizer";
