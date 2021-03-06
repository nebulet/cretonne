//! Instruction shrinking.
//!
//! Sometimes there are multiple valid encodings for a given instruction. Cretonne often initially
//! chooses the largest one, because this typically provides the register allocator the most
//! flexibility. However, once register allocation is done, this is no longer important, and we
//! can switch to smaller encodings when possible.

use ir::Function;
use isa::TargetIsa;
use regalloc::RegDiversions;
use timing;

/// Pick the smallest valid encodings for instructions.
pub fn shrink_instructions(func: &mut Function, isa: &TargetIsa) {
    let _tt = timing::shrink_instructions();

    let encinfo = isa.encoding_info();
    let mut divert = RegDiversions::new();

    for ebb in func.layout.ebbs() {
        divert.clear();
        for inst in func.layout.ebb_insts(ebb) {
            let enc = func.encodings[inst];
            if enc.is_legal() {
                let ctrl_type = func.dfg.ctrl_typevar(inst);

                // Pick the last encoding with constraints that are satisfied.
                let best_enc = isa.legal_encodings(func, &func.dfg[inst], ctrl_type)
                    .filter(|e| encinfo.constraints[e.recipe()].satisfied(inst, &divert, &func))
                    .min_by_key(|e| encinfo.bytes(*e))
                    .unwrap();

                if best_enc != enc {
                    func.encodings[inst] = best_enc;

                    dbg!(
                        "Shrunk [{}] to [{}] in {}, reducing the size from {} to {}",
                        encinfo.display(enc),
                        encinfo.display(best_enc),
                        func.dfg.display_inst(inst, isa),
                        encinfo.bytes(enc),
                        encinfo.bytes(best_enc)
                    );
                }
            }
            divert.apply(&func.dfg[inst]);
        }
    }
}
