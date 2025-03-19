#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <exception>
#include <new>
#include <utility>

#include <stdint.h>

struct out_of_stack : std::exception {
    const char *what() const noexcept override
    {
        return "out of Lua stack";
    }
};

extern "C" lua_State *engine_new()
{
    // Create Lua state.
    auto L = luaL_newstate();

    if (!L) {
        throw std::bad_alloc();
    }

    // Register libraries that does not need to alter its behavior.
    auto libs = {
        std::make_pair(LUA_GNAME, luaopen_base),
        std::make_pair(LUA_COLIBNAME, luaopen_coroutine),
        std::make_pair(LUA_TABLIBNAME, luaopen_table),
        std::make_pair(LUA_IOLIBNAME, luaopen_io),
        std::make_pair(LUA_STRLIBNAME, luaopen_string),
        std::make_pair(LUA_MATHLIBNAME, luaopen_math),
        std::make_pair(LUA_UTF8LIBNAME, luaopen_utf8)
    };

    for (auto &l : libs) {
        luaL_requiref(L, l.first, l.second, 1);
        lua_pop(L, 1);
    }

    // Register "os" library.
    luaL_requiref(L, LUA_OSLIBNAME, luaopen_os, 1);

    if (!lua_checkstack(L, 1)) {
        lua_close(L);
        throw out_of_stack();
    }

    lua_pushnil(L);
    lua_setfield(L, -2, "exit");
    lua_pushnil(L);
    lua_setfield(L, -2, "setlocale");
    lua_pop(L, 1);

    return L;
}

extern "C" void engine_free(lua_State *L)
{
    lua_close(L);
}

extern "C" bool engine_load(lua_State *L, const char *name, const char *script, size_t len)
{
    if (!lua_checkstack(L, 1)) {
        throw out_of_stack();
    }

    return luaL_loadbufferx(L, script, len, name, "t") == LUA_OK;
}

extern "C" bool engine_pcall(lua_State *L, int nargs, int nresults, int msgh)
{
    return lua_pcall(L, nargs, nresults, msgh) == LUA_OK;
}

extern "C" bool engine_isnil(lua_State *L, int index)
{
    return lua_isnil(L, index) != 0;
}

extern "C" int64_t engine_tointegerx(lua_State *L, int index, int *isnum)
{
    return static_cast<int64_t>(lua_tointegerx(L, index, isnum));
}

extern "C" const char *engine_tostring(lua_State *L, int index)
{
    return lua_tostring(L, index);
}

extern "C" const char *engine_typename(lua_State *L, int index)
{
    return lua_typename(L, lua_type(L, index));
}

extern "C" void engine_pop(lua_State *L, int n)
{
    lua_pop(L, n);
}
