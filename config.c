#include <stdio.h>
#include <luaconf.h>
#include <lua.h>

#define STRINGIFY(s) #s
#define STR(s) STRINGIFY(s)

int main(int argc, char **argv) {
	const char *libname = "lua";
	if (argc > 1) libname = argv[1];

	printf("//! Module for configuration based on luaconf.h\n\n");
	printf("use std::libc;\n\n");

	printf("#[link(name = \"%s\")]\n", libname);
	printf("extern {}\n\n");

	printf("/// Human-readable major version string\n");
	printf("pub static LUA_VERSION: &'static str = \"%s\";\n", LUA_VERSION);
	printf("/// Human-readable release version string\n");
	printf("pub static LUA_RELEASE: &'static str = \"%s\";\n", LUA_RELEASE);
	printf("/// Machine-readable Lua version number\n");
	printf("pub static LUA_VERSION_NUM: libc::c_int = %d;\n\n", LUA_VERSION_NUM);

	printf("/// The integral type used by lua_pushinteger/lua_tointeger.\n");
	printf("pub type LUA_INTEGER = libc::" STR(LUA_INTEGER) ";\n");
	printf("/// The type of numbers in Lua.\n");
	printf("pub type LUA_NUMBER = libc::c_" STR(LUA_NUMBER) ";\n\n");

	printf("/// LUA_QL describes how error messages quote program elements.\n");
	printf("pub static LUA_QL: &'static str = \"" LUA_QL("{}") "\";\n\n");

	printf("/// The buffer size used by the lauxlib buffer system.\n");
	printf("pub static LUAL_BUFFERSIZE: libc::size_t = %d;\n\n", LUAL_BUFFERSIZE);

	printf("/// The maximum size for the description of the source of a function in debug information.\n");
	printf("pub static LUA_IDSIZE: libc::size_t = %d;\n\n", LUA_IDSIZE);

	// include LUA_MINSTACK here even though it's not in luaconf.h
	printf("/// The minimum Lua stack available to a C function.\n");
	printf("pub static LUA_MINSTACK: libc::size_t = %d;\n", LUA_MINSTACK);

	return 0;
}
