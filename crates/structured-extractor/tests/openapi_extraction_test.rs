//! # OpenAPI and Swagger Specification Extraction Tests
//!
//! Validates OpenAPI 3.0, OpenAPI 3.1, and Swagger 2.0 extraction across JSON and YAML formats,
//! testing operation parameter tables, request body payloads, response status codes,
//! component schema data models, tag grouping, and CDM document synthesis.

use web_repo_to_ai_cdm::SourceType;
use web_repo_to_ai_structured_extractor::{
    detect_format, parse_openapi, StructuredDocumentKind, StructuredExtractor, StructuredInput,
    StructuredOptions,
};

const SAMPLE_OPENAPI_YAML: &str = r##"
openapi: 3.0.3
info:
  title: Petstore Inventory API
  version: 2.4.0
  description: High-performance pet store inventory and tracking service.
servers:
  - url: https://api.petstore.example.com/v2
    description: Production Cluster
  - url: https://staging.petstore.example.com/v2
    description: Staging Environment
tags:
  - name: Pets
    description: Pet inventory and adoption operations
  - name: Orders
    description: Store orders and fulfillment
paths:
  /pets:
    get:
      summary: List all pets
      description: Returns a paginated catalog of all pets available for adoption.
      operationId: listPets
      tags:
        - Pets
      parameters:
        - name: limit
          in: query
          required: false
          description: Maximum number of pets to return
          schema:
            type: integer
            format: int32
        - name: status
          in: query
          required: false
          description: Filter pets by status
          schema:
            type: string
            enum: [available, pending, adopted]
      responses:
        "200":
          description: A list of pets
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/Pet"
        "400":
          description: Invalid request parameters
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ErrorResponse"
    post:
      summary: Add a new pet
      description: Registers a new pet in the adoption catalog.
      operationId: addPet
      tags:
        - Pets
      requestBody:
        description: Pet details to register
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/NewPet"
      responses:
        "201":
          description: Pet created successfully
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Pet"
  /pets/{pet_id}:
    get:
      summary: Get pet by ID
      operationId: getPetById
      tags:
        - Pets
      parameters:
        - name: pet_id
          in: path
          required: true
          description: Unique pet identifier
          schema:
            type: string
            format: uuid
      responses:
        "200":
          description: Pet details
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Pet"
        "404":
          description: Pet not found
components:
  schemas:
    Pet:
      type: object
      required:
        - id
        - name
      properties:
        id:
          type: string
          format: uuid
          description: Unique pet identifier
        name:
          type: string
          description: Pet given name
        tag:
          type: string
          description: Pet species or category tag
        age:
          type: integer
          description: Estimated age in years
    NewPet:
      type: object
      required:
        - name
      properties:
        name:
          type: string
          description: Pet name
        tag:
          type: string
    ErrorResponse:
      type: object
      required:
        - code
        - message
      properties:
        code:
          type: integer
          format: int32
        message:
          type: string
"##;

const SAMPLE_SWAGGER_JSON: &str = r##"{
  "swagger": "2.0",
  "info": {
    "title": "Legacy Gateway API",
    "version": "1.1.0",
    "description": "Legacy microservice gateway service"
  },
  "host": "gateway.internal.net",
  "basePath": "/v1",
  "schemes": ["https"],
  "paths": {
    "/auth/login": {
      "post": {
        "summary": "Authenticate user",
        "description": "Issues authentication session token",
        "operationId": "loginUser",
        "parameters": [
          {
            "name": "body",
            "in": "body",
            "required": true,
            "description": "User login credentials",
            "schema": {
              "$ref": "#/definitions/LoginPayload"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "Login successful",
            "schema": {
              "$ref": "#/definitions/AuthToken"
            }
          },
          "401": {
            "description": "Invalid credentials"
          }
        }
      }
    }
  },
  "definitions": {
    "LoginPayload": {
      "type": "object",
      "required": ["username", "password"],
      "properties": {
        "username": {
          "type": "string"
        },
        "password": {
          "type": "string",
          "format": "password"
        }
      }
    },
    "AuthToken": {
      "type": "object",
      "required": ["token", "expires_in"],
      "properties": {
        "token": {
          "type": "string"
        },
        "expires_in": {
          "type": "integer"
        }
      }
    }
  }
}"##;

#[test]
fn test_openapi_yaml_parsing_and_detection() {
    let input = StructuredInput::from_str(SAMPLE_OPENAPI_YAML).with_filename("openapi.yaml");
    let kind = detect_format(&input).expect("detect format");
    assert_eq!(kind, StructuredDocumentKind::OpenApi);

    let spec = parse_openapi(SAMPLE_OPENAPI_YAML).expect("parse OpenAPI spec");
    assert_eq!(spec.spec_format_version, "3.0.3");
    assert_eq!(spec.title, "Petstore Inventory API");
    assert_eq!(spec.api_version, "2.4.0");
    assert_eq!(spec.servers.len(), 2);
    assert_eq!(spec.paths.len(), 2);
    assert_eq!(spec.schemas.len(), 3);
    assert_eq!(spec.tags.len(), 2);
}

#[test]
fn test_swagger_json_parsing_and_detection() {
    let input = StructuredInput::from_str(SAMPLE_SWAGGER_JSON).with_filename("swagger.json");
    let kind = detect_format(&input).expect("detect format");
    assert_eq!(kind, StructuredDocumentKind::OpenApi);

    let spec = parse_openapi(SAMPLE_SWAGGER_JSON).expect("parse Swagger spec");
    assert_eq!(spec.spec_format_version, "Swagger 2.0");
    assert_eq!(spec.title, "Legacy Gateway API");
    assert_eq!(spec.servers.len(), 1);
    assert_eq!(spec.servers[0], "https://gateway.internal.net/v1");

    assert_eq!(spec.paths.len(), 1);
    let op = &spec.paths[0].operations[0];
    assert_eq!(op.method, "POST");
    assert_eq!(op.operation_id.as_deref(), Some("loginUser"));
    assert!(op.request_body.is_some());
    assert_eq!(
        op.request_body.as_ref().unwrap().schema_summary.as_deref(),
        Some("LoginPayload")
    );

    assert_eq!(spec.schemas.len(), 2);
}

#[test]
fn test_openapi_cdm_extraction_and_sections() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(SAMPLE_OPENAPI_YAML)
        .with_filename("petstore.yaml")
        .with_source_url("https://petstore.example.com/openapi.yaml");
    let options = StructuredOptions::default();

    let cdm = extractor
        .extract_to_cdm(&input, &options)
        .expect("extract to CDM");

    assert_eq!(cdm.source_type, SourceType::StructuredDocument);
    assert_eq!(
        cdm.metadata.title.as_deref(),
        Some("Petstore Inventory API")
    );
    assert_eq!(cdm.metadata.extra["structured_format"], "openapi");
    assert_eq!(cdm.metadata.extra["spec_format_version"], "3.0.3");
    assert_eq!(cdm.metadata.extra["endpoint_count"], "3");
    assert_eq!(cdm.metadata.extra["schema_count"], "3");

    // Sections: Overview (1) + 3 Endpoints (3) + 3 Schemas (3) = 7 sections
    assert_eq!(cdm.sections.len(), 7);

    // Verify Overview section
    let overview_sec = &cdm.sections[0];
    assert!(overview_sec
        .title
        .as_deref()
        .unwrap_or("")
        .contains("Overview"));
    assert!(overview_sec.body.contains("Petstore Inventory API"));
    assert!(overview_sec
        .body
        .contains("https://api.petstore.example.com/v2"));

    // Verify Endpoint section with parameters table
    let list_pets_sec = cdm
        .sections
        .iter()
        .find(|s| s.title.as_deref().unwrap_or("").contains("GET /pets"))
        .expect("list pets section");
    assert!(list_pets_sec
        .body
        .contains("| `limit` | `query` | `integer (int32)` | No |"));
    assert!(list_pets_sec
        .body
        .contains("| `200` | A list of pets | `application/json` | `Array<Pet>` |"));

    // Verify Schema section
    let pet_schema_sec = cdm
        .sections
        .iter()
        .find(|s| s.title.as_deref().unwrap_or("").contains("Model: Pet"))
        .expect("Pet schema section");
    assert!(pet_schema_sec
        .body
        .contains("| `id` | `string` | Yes | `uuid` |"));
    assert!(pet_schema_sec
        .body
        .contains("| `name` | `string` | Yes | `-` |"));

    // Verify Hierarchy
    assert_eq!(cdm.hierarchy.name, "Petstore Inventory API");
    assert!(cdm.hierarchy.children.len() >= 2);
}

#[test]
fn test_openapi_tag_grouping_vs_path_grouping() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(SAMPLE_OPENAPI_YAML);

    // 1. Tag grouping (default)
    let opts_tag = StructuredOptions::default().with_group_by_tag(true);
    let cdm_tag = extractor
        .extract_to_cdm(&input, &opts_tag)
        .expect("tag grouping");
    let has_pets_tag = cdm_tag.hierarchy.children.iter().any(|c| c.name == "Pets");
    assert!(has_pets_tag);

    // 2. Path grouping
    let opts_path = StructuredOptions::default().with_group_by_tag(false);
    let cdm_path = extractor
        .extract_to_cdm(&input, &opts_path)
        .expect("path grouping");
    let has_endpoints_group = cdm_path
        .hierarchy
        .children
        .iter()
        .any(|c| c.name == "Endpoints");
    assert!(has_endpoints_group);
}
