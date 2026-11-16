# Rule 09: No Mock, No Placeholder, No Fake Implementation

1. **Zero Mock Data in Production Code:** Never ship hardcoded sample repositories, canned webpage conversions, fake progress timers, or fabricated token counts.
2. **Zero Placeholder Implementations:** Do not mark any subphase complete if it relies on stubbed functions returning dummy values, `todo!()`, or inactive UI buttons.
3. **Future Integrations Boundary:** Prepare clean Rust traits and extension interfaces for future stages (CLI, MCP, Browser Extension, RAG adapters), but never expose fake UI toggles or dead endpoints claiming those integrations are active before their stage is implemented.
