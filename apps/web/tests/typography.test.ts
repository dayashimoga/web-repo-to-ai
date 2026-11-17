import { describe, it, expect } from "vitest";
import { Heading, Text, Code, Mono } from "../src/lib/components/typography/index";
import { Container, Stack, Grid } from "../src/lib/components/layout/index";

describe("Typography & Layout Components Export Validation", () => {
  it("exports all typography components cleanly", () => {
    expect(Heading).toBeDefined();
    expect(Text).toBeDefined();
    expect(Code).toBeDefined();
    expect(Mono).toBeDefined();
  });

  it("exports all layout components cleanly", () => {
    expect(Container).toBeDefined();
    expect(Stack).toBeDefined();
    expect(Grid).toBeDefined();
  });
});
