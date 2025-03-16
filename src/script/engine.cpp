#include <lua.hpp>

#include <new>

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
