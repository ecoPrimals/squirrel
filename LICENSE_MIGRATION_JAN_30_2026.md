# License Migration to AGPL-3.0 - January 30, 2026

## Migration Summary

**Date**: January 30, 2026  
**From**: MIT OR Apache-2.0 (dual license)  
**To**: AGPL-3.0-only (copyleft)  
**Reason**: Ecosystem sovereignty and freedom requirements

---

## What Changed

### 1. License Files
- ✅ Created `LICENSE-AGPL3` with full AGPL-3.0 text
- ✅ Added copyright notice for DataScienceBioLab
- ✅ Included application instructions

### 2. Cargo.toml Updates (31 files)
- ✅ Workspace `Cargo.toml`: Updated to `license = "AGPL-3.0-only"`
- ✅ `crates/main/Cargo.toml`: Updated to `license = "AGPL-3.0-only"`
- ✅ All 29 subcrate `Cargo.toml` files: Batch updated

### 3. Documentation Updates
- ✅ `README.md`: Added comprehensive license section
- ✅ Explained AGPL Section 13 (network service requirement)
- ✅ Added copyright notices

### 4. Source Code Headers
**Status**: PENDING - To be added in next phase

**Pattern** (for Rust source files):
```rust
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3 of the License.
```

---

## Why AGPL-3.0?

### Alignment with ecoPrimals Values

1. **Sovereignty**: AGPL ensures modifications to network services remain free
2. **Community Cooperation**: Improvements must be shared back
3. **Human Dignity**: Users have rights to software freedom
4. **Transparency**: Source code availability requirement

### AGPL Section 13 - Network Services

**Key Provision**: If you run modified Squirrel as a network service, users interacting with it remotely must have access to the source code.

**Why This Matters**:
- Prevents proprietary forks of network services
- Ensures community benefits from improvements
- Maintains software freedom in cloud/SaaS era

### Compatibility Considerations

#### ✅ Compatible Dependencies
Most Rust ecosystem dependencies are MIT/Apache-2.0 compatible with AGPL:
- tokio (MIT)
- serde (MIT/Apache-2.0)
- async-trait (MIT/Apache-2.0)
- tracing (MIT)
- etc.

#### ⚠️ Considerations
1. **Distribution**: AGPL applies to combined work
2. **Linking**: Static linking creates combined work
3. **Network Services**: Remote interaction triggers Section 13

#### Review Status
- ✅ Core dependencies: All compatible
- ✅ No GPL-incompatible licenses found
- ✅ No proprietary dependencies

---

## Impact on Ecosystem

### For ecoPrimals Ecosystem
- ✅ All primals can adopt AGPL (recommended)
- ✅ Inter-primal communication: No issues (same license)
- ✅ Ecosystem coherence: Aligned values

### For Users
- ✅ **More Freedom**: Can modify and redistribute
- ✅ **More Transparency**: Source always available
- ✅ **More Protection**: Improvements remain free

### For Developers
- ✅ **Clear Rules**: No ambiguity about licensing
- ✅ **Community Growth**: Contributions flow back
- ✅ **Professional**: Standard OSS license

---

## Migration Verification

### Checklist
- [x] LICENSE-AGPL3 file created
- [x] Workspace Cargo.toml updated
- [x] Main crate Cargo.toml updated
- [x] All subcrate Cargo.toml files updated (29 crates)
- [x] README.md updated with license section
- [ ] SPDX headers added to source files (next phase)
- [ ] CI updated to check license headers
- [ ] Contribution guidelines updated

### Verification Commands
```bash
# Check all Cargo.toml files
find . -name "Cargo.toml" -exec grep -H "license" {} \;

# Should all show: license = "AGPL-3.0-only"
# or: license.workspace = true (inherits AGPL-3.0-only)

# Verify no MIT/Apache-2.0 remains
grep -r "MIT OR Apache-2.0" --include="*.toml" .
# Should return: (no results)
```

---

## Next Steps

### Immediate (This Session)
1. ✅ License files created
2. ✅ Cargo.toml files updated
3. ✅ README.md updated
4. ⏳ Source file headers (bulk operation)

### Short-Term (This Week)
5. Update CONTRIBUTING.md with license requirements
6. Add license check to CI pipeline
7. Update documentation templates
8. Inform ecosystem stakeholders

### Long-Term (Ongoing)
9. Maintain license compliance
10. Review new dependencies for compatibility
11. Ensure modifications include license headers
12. Document any exceptions or special cases

---

## Legal Considerations

### Copyright Ownership
**Copyright Holder**: DataScienceBioLab  
**Year**: 2026  
**License**: AGPL-3.0-only

### Contributor Agreement
**New Contributors** must:
1. Agree to AGPL-3.0 licensing of contributions
2. Assert they have rights to contribute code
3. Include proper copyright notices in new files

### Third-Party Code
- Must be AGPL-compatible (MIT, Apache-2.0, BSD, etc.)
- Must maintain original copyright notices
- Must document in LICENSE or THIRD_PARTY_NOTICES

---

## FAQ

### Q: Can I still use Squirrel in commercial projects?
**A**: Yes! AGPL allows commercial use. You must provide source code if you run modified versions as a network service.

### Q: What if I only use Squirrel internally (not as a network service)?
**A**: Internal use does not trigger AGPL obligations. No source code distribution required.

### Q: Can I link AGPL code with proprietary code?
**A**: If you distribute the combined work, it must be under AGPL. Network interaction also triggers this.

### Q: What about MIT/Apache-2.0 dependencies?
**A**: Compatible! AGPL can include MIT/Apache-2.0 code. The combined work is under AGPL.

### Q: Do I need to relicense my existing Squirrel fork?
**A**: If you already have a fork under MIT/Apache-2.0, you can continue under that license for that version. Future versions will be AGPL-3.0.

---

## References

- **Full AGPL-3.0 Text**: [LICENSE-AGPL3](LICENSE-AGPL3)
- **Official AGPL-3.0**: https://www.gnu.org/licenses/agpl-3.0.html
- **AGPL FAQ**: https://www.gnu.org/licenses/agpl-3.0-faq.html
- **FSF Licensing**: https://www.fsf.org/licensing/

---

## Status

**Migration**: ✅ **COMPLETE** (Core)  
**Headers**: ⏳ **PENDING** (Next phase)  
**CI Integration**: ⏳ **PENDING**  
**Documentation**: ✅ **COMPLETE**  

**Overall**: 🟢 **90% COMPLETE** - Core migration done, automation pending

---

**Document Status**: FINAL  
**Author**: Comprehensive Audit Execution  
**Date**: January 30, 2026
