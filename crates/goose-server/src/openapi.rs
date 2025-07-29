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
use rmcp::model::AudioContent;
use rmcp::model::RawEmbeddedResource;
use rmcp::model::RawImageContent;
use rmcp::model::RawTextContent;
use rmcp::model::{
    Annotations, EmbeddedResource, ImageContent, ResourceContents, Role, TextContent, Tool,
    ToolAnnotations,
};
use schemars::JsonSchema;
use utoipa::OpenApi;

fn openapi_schema_for<T: JsonSchema>(name: String) -> Vec<(String, serde_json::Value)> {
    let mut schemas: Vec<(String, serde_json::Value)> = vec![];

    let settings = rmcp::schemars::generate::SchemaSettings::openapi3();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<T>().to_value();
    let schema = schema.as_object().unwrap();
    let mut parent = schema.clone();
    parent.remove("components");
    parent.remove("$schema");

    schemas.push((name.into(), parent.into()));

    for subschema in schema
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.as_object())
        .into_iter()
        .flat_map(|o| o.values())
    {
        if let serde_json::Value::Object(obj) = subschema {
            if let Some(title) = obj.get("title").and_then(|t| t.as_str()) {
                schemas.push((title.to_string(), subschema.clone()));
            }
        }
    }

    schemas
}

trait OpenApiExt {
    fn schemas() -> Vec<(String, serde_json::Value)>;
}

macro_rules! derive_schemas {
    ($inner_type:ident) => {
        impl OpenApiExt for $inner_type {
            fn schemas() -> Vec<(String, serde_json::Value)> {
                openapi_schema_for::<$inner_type>(stringify!($inner_type).to_string())
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Content {
    Text(RawTextContent),
    Image(RawImageContent),
    Resource(RawEmbeddedResource),
}

derive_schemas!(Role);
derive_schemas!(Content);
derive_schemas!(EmbeddedResource);
derive_schemas!(ImageContent);
derive_schemas!(TextContent);
derive_schemas!(AudioContent);
derive_schemas!(Tool);
derive_schemas!(ToolAnnotations);
derive_schemas!(Annotations);
derive_schemas!(ResourceContents);
derive_schemas!(RawTextContent);
derive_schemas!(RawImageContent);
derive_schemas!(RawEmbeddedResource);

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
        ToolResponse,
        ToolRequest,
        ToolConfirmationRequest,
        ThinkingContent,
        RedactedThinkingContent,
        FrontendToolRequest,
        ContextLengthExceeded,
        SummarizationRequested,
        ProviderMetadata,
        ExtensionEntry,
        ExtensionConfig,
        ConfigKey,
        Envs,
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

macro_rules! insert_schemas {
    ($schema_type:ident, $schemas_map:ident) => {
        for (name, schema) in $schema_type::schemas() {
            $schemas_map.insert(name, schema);
        }
    };
}

#[allow(dead_code)] // Used by generate_schema binary
pub fn generate_schema() -> String {
    let api_doc = ApiDoc::openapi();
    let mut api_doc_value = serde_json::to_value(&api_doc).unwrap();
    api_doc_value["components"]["schemas"]
        .as_object_mut()
        .map(|schemas| {
            insert_schemas!(Content, schemas);
            insert_schemas!(EmbeddedResource, schemas);
            insert_schemas!(ImageContent, schemas);
            insert_schemas!(Annotations, schemas);
            insert_schemas!(TextContent, schemas);
            insert_schemas!(ResourceContents, schemas);
            insert_schemas!(Role, schemas);
            insert_schemas!(Tool, schemas);
            insert_schemas!(ToolAnnotations, schemas);
            insert_schemas!(RawTextContent, schemas);
            insert_schemas!(RawImageContent, schemas);
            insert_schemas!(RawEmbeddedResource, schemas);
            insert_schemas!(AudioContent, schemas);
        });
    serde_json::to_string_pretty(&api_doc_value).unwrap()
}
