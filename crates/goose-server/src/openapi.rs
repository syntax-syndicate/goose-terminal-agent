use goose::agents::extension::Envs;
use goose::agents::extension::ToolInfo;
use goose::agents::ExtensionConfig;
use goose::config::permission::PermissionLevel;
use goose::config::ExtensionEntry;
use goose::message::{
    ContextLengthExceeded, FrontendToolRequest, Message, MessageContent, RedactedThinkingContent,
    SummarizationRequested, ThinkingContent, ToolConfirmationRequest, ToolRequest, ToolResponse,
};
use goose::permission::permission_confirmation::PrincipalType;
use goose::providers::base::{ConfigKey, ModelInfo, ProviderMetadata};
use goose::session::info::SessionInfo;
use goose::session::SessionMetadata;
use rmcp::model::Annotations;
use rmcp::model::Content;
use rmcp::model::EmbeddedResource;
use rmcp::model::ImageContent;
use rmcp::model::ResourceContents;
use rmcp::model::Role;
use rmcp::model::TextContent;
use rmcp::model::Tool;
use rmcp::model::ToolAnnotations;
use serde_json::Map;
use utoipa::openapi::schema::AdditionalProperties;
use utoipa::openapi::schema::AnyOfBuilder;
use utoipa::openapi::AllOfBuilder;
use utoipa::openapi::ArrayBuilder;
use utoipa::openapi::ObjectBuilder;
use utoipa::openapi::OneOfBuilder;
use utoipa::openapi::Ref;
use utoipa::openapi::SchemaFormat;
use utoipa::openapi::SchemaType;
use utoipa::{OpenApi, ToSchema};

use utoipa::openapi::RefOr;

// use rmcp::schemars::schema::Schema;
use utoipa::openapi::schema::Schema;

macro_rules! derive_utoipa {
    ($inner_type:ident as $schema_name:ident) => {
        struct $schema_name {}

        impl<'__s> ToSchema<'__s> for $schema_name {
            fn schema() -> (
                &'__s str,
                utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
            ) {
                let settings = rmcp::schemars::generate::SchemaSettings::openapi3();
                let schemars_schema = settings.into_generator().root_schema_for::<$inner_type>();
                (
                    stringify!($inner_type),
                    RefOr::T(from_json(schemars_schema.to_value())),
                )
            }
        }
    };
}

use serde_json::Value;
use utoipa::openapi::schema::{AllOf, AnyOf, Array, Object, OneOf};

fn from_json_to_refor(value: serde_json::Value) -> RefOr<Schema> {
    match &value {
        Value::Object(map) => {
            // Check if this has both $ref and other properties (like properties, required, type)
            if let Some(ref_value) = map.get("$ref").and_then(|v| v.as_str()) {
                let has_other_properties = map.keys().any(|k| k != "$ref");

                if has_other_properties {
                    // This has both $ref and other properties - create AllOf
                    let mut all_of_schema = AllOf::new();

                    // First item: the reference
                    all_of_schema.items.push(RefOr::Ref(Ref::new(ref_value)));

                    // Second item: the object without $ref
                    let mut object_map = map.clone();
                    object_map.remove("$ref");
                    let object_schema = from_json(Value::Object(object_map));
                    all_of_schema.items.push(RefOr::T(object_schema));

                    RefOr::T(Schema::AllOf(all_of_schema))
                } else {
                    // Just a reference
                    RefOr::Ref(Ref::new(ref_value))
                }
            } else {
                RefOr::T(from_json(value))
            }
        }
        _ => RefOr::T(from_json(value)),
    }
}

fn from_json(value: serde_json::Value) -> Schema {
    match value {
        Value::Object(map) => {
            // Handle composite schemas - but check if this is a mixed object+oneOf case
            if let Some(one_of) = map.get("oneOf").and_then(|v| v.as_array()) {
                // Check if this object has other properties besides oneOf
                let has_other_properties = map
                    .keys()
                    .any(|k| k != "oneOf" && k != "$schema" && k != "title" && k != "$defs");

                if has_other_properties {
                    // This is an object with oneOf - create AllOf with object + oneOf
                    let mut all_of_schema = AllOf::new();

                    // First item: the object without oneOf
                    let mut object_map = map.clone();
                    object_map.remove("oneOf");
                    object_map.remove("$schema");
                    object_map.remove("title");
                    object_map.remove("$defs");
                    let object_schema = from_json(Value::Object(object_map));
                    all_of_schema.items.push(RefOr::T(object_schema));

                    // Second item: the oneOf
                    let mut one_of_schema = OneOf::new();
                    for schema_value in one_of {
                        let ref_or_schema = from_json_to_refor(schema_value.clone());
                        one_of_schema.items.push(ref_or_schema);
                    }
                    all_of_schema
                        .items
                        .push(RefOr::T(Schema::OneOf(one_of_schema)));

                    Schema::AllOf(all_of_schema)
                } else {
                    // Simple oneOf without other properties
                    let mut one_of_schema = OneOf::new();
                    for schema_value in one_of {
                        let ref_or_schema = from_json_to_refor(schema_value.clone());
                        one_of_schema.items.push(ref_or_schema);
                    }
                    Schema::OneOf(one_of_schema)
                }
            } else if let Some(all_of) = map.get("allOf").and_then(|v| v.as_array()) {
                let mut all_of_schema = AllOf::new();
                for schema_value in all_of {
                    let ref_or_schema = from_json_to_refor(schema_value.clone());
                    all_of_schema.items.push(ref_or_schema);
                }
                Schema::AllOf(all_of_schema)
            } else if let Some(any_of) = map.get("anyOf").and_then(|v| v.as_array()) {
                let mut any_of_schema = AnyOf::new();
                for schema_value in any_of {
                    let ref_or_schema = from_json_to_refor(schema_value.clone());
                    any_of_schema.items.push(ref_or_schema);
                }
                Schema::AnyOf(any_of_schema)
            }
            // Check for schema type
            else if let Some(schema_type) = map.get("type") {
                // Handle union types (array of types)
                if let Some(type_array) = schema_type.as_array() {
                    let mut any_of_schema = AnyOf::new();
                    for type_value in type_array {
                        if let Some(type_str) = type_value.as_str() {
                            let mut type_map = serde_json::Map::new();
                            type_map
                                .insert("type".to_string(), Value::String(type_str.to_string()));

                            // Copy other properties from original map except "type"
                            for (key, value) in &map {
                                if key != "type" {
                                    type_map.insert(key.clone(), value.clone());
                                }
                            }

                            let type_schema = from_json(Value::Object(type_map));
                            any_of_schema.items.push(RefOr::T(type_schema));
                        }
                    }
                    Schema::AnyOf(any_of_schema)
                } else {
                    // Single type
                    match schema_type.as_str() {
                        Some("array") => {
                            // Create items schema
                            let items_ref_or = if let Some(items) = map.get("items") {
                                from_json_to_refor(items.clone())
                            } else {
                                RefOr::T(Schema::Object(Object::new()))
                            };

                            let mut array = Array::new(RefOr::T(Schema::Object(Object::new())));
                            array.items = Box::new(items_ref_or);

                            // Handle min/max items
                            if let Some(min_items) = map.get("minItems").and_then(|v| v.as_u64()) {
                                array.min_items = Some(min_items as usize);
                            }
                            if let Some(max_items) = map.get("maxItems").and_then(|v| v.as_u64()) {
                                array.max_items = Some(max_items as usize);
                            }

                            Schema::Array(array)
                        }
                        Some("object") => {
                            let mut object = Object::new();

                            // Handle properties
                            if let Some(properties) =
                                map.get("properties").and_then(|v| v.as_object())
                            {
                                for (prop_name, prop_schema) in properties {
                                    let ref_or_schema = from_json_to_refor(prop_schema.clone());
                                    object.properties.insert(prop_name.clone(), ref_or_schema);
                                }
                            }

                            // Handle required properties
                            if let Some(required) = map.get("required").and_then(|v| v.as_array()) {
                                for req in required {
                                    if let Some(req_str) = req.as_str() {
                                        object.required.push(req_str.to_string());
                                    }
                                }
                            }

                            Schema::Object(object)
                        }
                        Some("string") => {
                            let mut object = Object::with_type(SchemaType::String);

                            // Handle string constraints
                            if let Some(min_length) = map.get("minLength").and_then(|v| v.as_u64())
                            {
                                object.min_length = Some(min_length as usize);
                            }
                            if let Some(max_length) = map.get("maxLength").and_then(|v| v.as_u64())
                            {
                                object.max_length = Some(max_length as usize);
                            }
                            if let Some(pattern) = map.get("pattern").and_then(|v| v.as_str()) {
                                object.pattern = Some(pattern.to_string());
                            }

                            // Handle const values
                            if let Some(const_value) = map.get("const") {
                                // For const string values, we can set enum with single value
                                if let Some(const_str) = const_value.as_str() {
                                    object.enum_values = Some(vec![const_str.into()]);
                                }
                            }

                            Schema::Object(object)
                        }
                        Some("number") | Some("integer") => {
                            let schema_type = if schema_type.as_str() == Some("integer") {
                                SchemaType::Integer
                            } else {
                                SchemaType::Number
                            };

                            let mut object = Object::with_type(schema_type);

                            // Handle numeric constraints
                            if let Some(minimum) = map.get("minimum").and_then(|v| v.as_f64()) {
                                object.minimum = Some(minimum);
                            }
                            if let Some(maximum) = map.get("maximum").and_then(|v| v.as_f64()) {
                                object.maximum = Some(maximum);
                            }

                            Schema::Object(object)
                        }
                        Some("boolean") => Schema::Object(Object::with_type(SchemaType::Boolean)),
                        _ => Schema::Object(Object::new()),
                    }
                }
            } else {
                // Default to object schema
                Schema::Object(Object::new())
            }
        }
        _ => {
            // For non-object values, create a basic object schema
            Schema::Object(Object::new())
        }
    }
}

derive_utoipa!(Role as RoleSchema);
derive_utoipa!(Content as ContentSchema);
derive_utoipa!(EmbeddedResource as EmbeddedResourceSchema);
derive_utoipa!(ImageContent as ImageContentSchema);
derive_utoipa!(TextContent as TextContentSchema);
derive_utoipa!(Tool as ToolSchema);
derive_utoipa!(ToolAnnotations as ToolAnnotationsSchema);
derive_utoipa!(Annotations as AnnotationsSchema);
derive_utoipa!(ResourceContents as ResourceContentsSchema);

#[allow(dead_code)] // Used by utoipa for OpenAPI generation
#[derive(OpenApi)]
#[openapi(
    paths(
        super::routes::config_management::backup_config,
        super::routes::config_management::recover_config,
        super::routes::config_management::validate_config,
        super::routes::config_management::init_config,
        super::routes::config_management::upsert_config,
        super::routes::config_management::remove_config,
        super::routes::config_management::read_config,
        super::routes::config_management::add_extension,
        super::routes::config_management::remove_extension,
        super::routes::config_management::get_extensions,
        super::routes::config_management::read_all_config,
        super::routes::config_management::providers,
        super::routes::config_management::upsert_permissions,
        super::routes::agent::get_tools,
        super::routes::agent::add_sub_recipes,
        super::routes::reply::confirm_permission,
        super::routes::context::manage_context,
        super::routes::session::list_sessions,
        super::routes::session::get_session_history,
        super::routes::schedule::create_schedule,
        super::routes::schedule::list_schedules,
        super::routes::schedule::delete_schedule,
        super::routes::schedule::update_schedule,
        super::routes::schedule::run_now_handler,
        super::routes::schedule::pause_schedule,
        super::routes::schedule::unpause_schedule,
        super::routes::schedule::kill_running_job,
        super::routes::schedule::inspect_running_job,
        super::routes::schedule::sessions_handler,
        super::routes::recipe::create_recipe,
        super::routes::recipe::encode_recipe,
        super::routes::recipe::decode_recipe
    ),
    components(schemas(
        super::routes::config_management::UpsertConfigQuery,
        super::routes::config_management::ConfigKeyQuery,
        super::routes::config_management::ConfigResponse,
        super::routes::config_management::ProvidersResponse,
        super::routes::config_management::ProviderDetails,
        super::routes::config_management::ExtensionResponse,
        super::routes::config_management::ExtensionQuery,
        super::routes::config_management::ToolPermission,
        super::routes::config_management::UpsertPermissionsQuery,
        super::routes::reply::PermissionConfirmationRequest,
        super::routes::context::ContextManageRequest,
        super::routes::context::ContextManageResponse,
        super::routes::session::SessionListResponse,
        super::routes::session::SessionHistoryResponse,
        Message,
        MessageContent,
        ContentSchema,
        EmbeddedResourceSchema,
        ImageContentSchema,
        AnnotationsSchema,
        TextContentSchema,
        ToolResponse,
        ToolRequest,
        ToolConfirmationRequest,
        ThinkingContent,
        RedactedThinkingContent,
        FrontendToolRequest,
        ResourceContentsSchema,
        ContextLengthExceeded,
        SummarizationRequested,
        RoleSchema,
        ProviderMetadata,
        ExtensionEntry,
        ExtensionConfig,
        ConfigKey,
        Envs,
        ToolSchema,
        ToolAnnotationsSchema,
        ToolInfo,
        PermissionLevel,
        PrincipalType,
        ModelInfo,
        SessionInfo,
        SessionMetadata,
        super::routes::schedule::CreateScheduleRequest,
        super::routes::schedule::UpdateScheduleRequest,
        super::routes::schedule::KillJobResponse,
        super::routes::schedule::InspectJobResponse,
        goose::scheduler::ScheduledJob,
        super::routes::schedule::RunNowResponse,
        super::routes::schedule::ListSchedulesResponse,
        super::routes::schedule::SessionsQuery,
        super::routes::schedule::SessionDisplayInfo,
        super::routes::recipe::CreateRecipeRequest,
        super::routes::recipe::AuthorRequest,
        super::routes::recipe::CreateRecipeResponse,
        super::routes::recipe::EncodeRecipeRequest,
        super::routes::recipe::EncodeRecipeResponse,
        super::routes::recipe::DecodeRecipeRequest,
        super::routes::recipe::DecodeRecipeResponse,
        goose::recipe::Recipe,
        goose::recipe::Author,
        goose::recipe::Settings,
        goose::recipe::RecipeParameter,
        goose::recipe::RecipeParameterInputType,
        goose::recipe::RecipeParameterRequirement,
        goose::recipe::Response,
        goose::recipe::SubRecipe,
        goose::agents::types::RetryConfig,
        goose::agents::types::SuccessCheck,
        super::routes::agent::AddSubRecipesRequest,
        super::routes::agent::AddSubRecipesResponse,
    ))
)]
pub struct ApiDoc;

#[allow(dead_code)] // Used by generate_schema binary
pub fn generate_schema() -> String {
    let api_doc = ApiDoc::openapi();
    serde_json::to_string_pretty(&api_doc).unwrap()
}
