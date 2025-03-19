# Nablo UI Roadmap

## Performance Optimization

- [ ] Introduce incremental command buffer update by using rendering tree
- [ ] Introduce tile-based rendering
- [x] More precise window event handling based on rstar tree instead of traverse whole ui tree(configurable)
- [ ] Combine text texture uploading tasks
- [ ] Dynamically compile hotspot shape expresstions into wgsl or texture (based on change frequency)
- [ ] Introduce compile optimize techniques such as constant propagation, dead code elimination, cse, e-graph, etc.
- [ ] Use multi-threading to improve performance
- [ ] Design a suitable memory management strategy between cpu and gpu to reduce memory usage
- [ ] Design a high-effiency encoding strategy for gpu to render shapes.

## Shader-related features

- [ ] Implement customizable shader system

## UI-related features

- [ ] Add more widgets