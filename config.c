#include <stdio.h>
#include <luaconf.h>

#define STRINGIFY(s) #s
#define STR(s) STRINGIFY(s)

int main() {
	printf("//! Module for configuration based on luaconf.h\n\n");
	printf("use std::libc;\n\n");

	printf("/// The integral type used by lua_pushinteger/lua_tointeger.\n");
	printf("pub type LUA_INTEGER = libc::" STR(LUA_INTEGER) ";\n");
	printf("/// The type of numbers in Lua.\n");
	printf("pub type LUA_NUMBER = libc::c_" STR(LUA_NUMBER) ";\n\n");

	printf("/// LUA_QL describes how error messages quote program elements.\n");
	printf("pub static LUA_QL: &'static str = \"" LUA_QL("{}") "\";\n\n");

	printf("/// The buffer size used by the lauxlib buffer system.\n");
	printf("pub static LUAL_BUFFERSIZE: libc::size_t = %d;\n", LUAL_BUFFERSIZE);

	return 0;
}
