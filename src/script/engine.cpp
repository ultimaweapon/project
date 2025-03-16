#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <exception>
#include <new>

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

extern "C" void engine_pop(lua_State *L, int n)
{
    lua_pop(L, n);
}

extern "C" bool engine_load(lua_State *L, const char *name, const char *script, size_t len)
{
    if (!lua_checkstack(L, 1)) {
        throw out_of_stack();
    }

    return luaL_loadbufferx(L, script, len, name, "t") == LUA_OK;
}

extern "C" const char *engine_to_string(lua_State *L, int index)
{
    return lua_tostring(L, index);
}
