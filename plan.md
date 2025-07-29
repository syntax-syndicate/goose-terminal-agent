ToolError -> ErrorData conversion

* We are converting usages of ToolError from the mcp_core crate to ErrorData from the rmcp crate
* Review the commit in 028c87c0dd8fed286eeaed881c9bec484d645c88 and then make a script capable of completing more conversions in the same way I did in that example, across all places needed in the codebase
* Have it only change direct callsites of ToolError
* Have it pay special attention to syntax, producing valid and syntactially correct rust code
* Have it Use good judgement when deciding what code to pass to ErrorData::new and make it capture the spirit of the original ToolError call - mapping messages about params to ErrorData with ErrorCode::INVALID_PARAMS for example
* When all references of ToolError are gone: make sure the program passes cargo format and cargo build
* Reset the state of the crates dir and do multiple runs if you need to
