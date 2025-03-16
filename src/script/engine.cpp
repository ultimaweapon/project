struct engine {};

extern "C" engine *engine_new()
{
    return new engine();
}

extern "C" void engine_free(engine *e)
{
    delete e;
}
