#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <new>
#include <type_traits>
#include <utility>

#include <stdint.h>

static_assert(sizeof(lua_Integer) == sizeof(int64_t));
static_assert(std::is_signed<lua_Integer>::value);

extern "C" lua_State *lua54_newstate()
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

    return L;
}

extern "C" void lua54_close(lua_State *L)
{
    lua_close(L);
}

extern "C" void engine_require_os(lua_State *L)
{
    luaL_requiref(L, LUA_OSLIBNAME, luaopen_os, 1);
}

extern "C" bool engine_load(lua_State *L, const char *name, const char *script, size_t len)
{
    return luaL_loadbufferx(L, script, len, name, "t") == LUA_OK;
}

extern "C" bool engine_pcall(lua_State *L, int nargs, int nresults, int msgh)
{
    return lua_pcall(L, nargs, nresults, msgh) == LUA_OK;
}

extern "C" void engine_checkstack(lua_State *L, int n)
{
    luaL_checkstack(L, n, nullptr);
}

extern "C" void engine_pushnil(lua_State *L)
{
    lua_pushnil(L);
}

extern "C" const char *engine_pushstring(lua_State *L, const char *s)
{
    return lua_pushstring(L, s);
}

extern "C" void engine_pushcclosure(lua_State *L, int (*fn) (lua_State *L), int n)
{
    lua_pushcclosure(L, fn, n);
}

extern "C" int engine_gettop(lua_State *L)
{
    return lua_gettop(L);
}

extern "C" const char *engine_checkstring(lua_State *L, int arg)
{
    return luaL_checkstring(L, arg);
}

extern "C" void lua54_typeerror(lua_State *L, int arg, const char *tname)
{
    luaL_typeerror(L, arg, tname);
}

extern "C" void engine_argerror(lua_State *L, int arg, const char *extramsg)
{
    luaL_argerror(L, arg, extramsg);
}

extern "C" bool engine_isnil(lua_State *L, int index)
{
    return lua_isnil(L, index) != 0;
}

extern "C" bool lua54_istable(lua_State *L, int index)
{
    return lua_istable(L, index) != 0;
}

extern "C" int64_t engine_tointegerx(lua_State *L, int index, int *isnum)
{
    return static_cast<int64_t>(lua_tointegerx(L, index, isnum));
}

extern "C" const char *engine_tostring(lua_State *L, int index)
{
    return lua_tostring(L, index);
}

extern "C" void *engine_touserdata(lua_State *L, int index)
{
    return lua_touserdata(L, index);
}

extern "C" int lua54_type(lua_State *L, int index)
{
    return lua_type(L, index);
}

extern "C" const char *lua54_typename(lua_State *L, int tp)
{
    return lua_typename(L, tp);
}

extern "C" void engine_createtable(lua_State *L, int narr, int nrec)
{
    lua_createtable(L, narr, nrec);
}

extern "C" int lua54_geti(lua_State *L, int index, int64_t i)
{
    return lua_geti(L, index, i);
}

extern "C" void lua54_seti(lua_State *L, int index, int64_t n)
{
    lua_seti(L, index, n);
}

extern "C" int lua54_getfield(lua_State *L, int index, const char *k)
{
    return lua_getfield(L, index, k);
}

extern "C" void engine_setfield(lua_State *L, int index, const char *k)
{
    lua_setfield(L, index, k);
}

extern "C" void *engine_newuserdatauv(lua_State *L, size_t size, int nuvalue)
{
    return lua_newuserdatauv(L, size, nuvalue);
}

extern "C" void engine_setmetatable(lua_State *L, int index)
{
    lua_setmetatable(L, index);
}

extern "C" int engine_upvalueindex(int i)
{
    return lua_upvalueindex(i);
}

extern "C" void lua54_setglobal(lua_State *L, const char *name)
{
    lua_setglobal(L, name);
}

extern "C" void lua54_replace(lua_State *L, int index)
{
    lua_replace(L, index);
}

extern "C" void engine_pop(lua_State *L, int n)
{
    lua_pop(L, n);
}

extern "C" int engine_error(lua_State *L, const char *msg)
{
    return luaL_error(L, "%s", msg);
}
