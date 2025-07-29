ToolError -> ErrorData conversion

* We are converting usages of ToolError from the mcp_core crate to ErrorData from the rmcp crate
* Review the commit in 028c87c0dd8fed286eeaed881c9bec484d645c88 and then complete more conversions in the same way I did in that example, across all places needed in the codebase
* There are a high number of these, so you will need to make a script to accomplish it quickly. Make sure the script only changes callsites of ToolError and replaces them with syntactically correct usages of ErrorData.
* Use good judgement when deciding what code to pass to ErrorData::new and make it capture the spirit of the original ToolError - bake this into the script
* Then make sure the program is syntactically correct by running cargo format
* Then make sure the program compiles by running cargo build
