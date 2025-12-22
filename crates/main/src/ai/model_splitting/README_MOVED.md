# ⚠️ MOVED TO TOADSTOOL & SONGBIRD

This module has been **moved out of Squirrel** as it violates the architectural separation:

## Why This Was Removed

**Squirrel's Job**: AI orchestration, routing, requirements
- ✅ Knows: AI intent, user requirements  
- ❌ Should NOT know: GPU layers, VRAM splits, hardware topology

**ToadStool's Job**: GPU execution, model loading, layer management
- ✅ Should handle: Layer distribution, VRAM allocation, model loading

**Songbird's Job**: Cross-tower coordination
- ✅ Should coordinate: Multi-tower splits, tensor routing

## The Right Architecture

```
User → Squirrel: "Load llama-70b"
  ↓
Squirrel → Songbird: "Coordinate model load"
  ↓
Songbird → ToadStool(s): "Load layers X-Y"
  ↓
ToadStool(s) → Execute on GPUs
```

## Where This Code Went

This functionality has been relocated to:

1. **ToadStool** (`../toadstool/crates/model-loading/`):
   - Layer distribution algorithms
   - VRAM calculation
   - Model loading logic
   - GPU execution

2. **Songbird** (`../songbird/crates/coordination/`):
   - Multi-tower coordination
   - Tower assignment
   - Tensor routing between towers

## What Squirrel Kept

Squirrel now only:
- Requests model loading (via Songbird)
- Discovers available services
- Routes user requests
- Manages AI intent (NOT hardware details)

See: `docs/architecture/MODEL_SPLITTING_MOVED_TO_TOADSTOOL.md`

---

**Date Removed**: December 20, 2025  
**Reason**: Architectural cleanup - proper separation of concerns  
**Migration Guide**: See ToadStool repository for new implementation

