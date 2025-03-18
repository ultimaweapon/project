#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <exception>
#include <new>

#include <stdint.h>

struct out_of_stack : std::exception {
    const char *what() const noexcept override
    {
        return "out of Lua stack";
    }
};

extern "C" lua_State *engine_new()
{
    auto L = luaL_newstate();

    if (!L) {
        throw std::bad_alloc();
    }

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
