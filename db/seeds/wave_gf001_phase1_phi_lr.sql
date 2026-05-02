-- ============================================================
-- Seed: Phase 1 — phi-LR ladder  (Quick-3 seeds × k=0..5)
-- alpha_phi = 6.18e-4,  lr(k) = alpha_phi * phi^(-k/2)
-- phi = 1.6180339887
-- seeds Quick-3: {34, 55, 89}  (Fibonacci)
-- ============================================================

-- k=0  lr ≈ 6.180e-4
INSERT INTO experiment_queue (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json) VALUES
 ('WAVE-GF-001-P1-k0-s34', 1, 2, 34, 6.180e-4, 'adamw', 256, 'gf16',
  '{"k":0,"phi_lr_k":0,"seed":34,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=0"}'),
 ('WAVE-GF-001-P1-k0-s55', 1, 2, 55, 6.180e-4, 'adamw', 256, 'gf16',
  '{"k":0,"phi_lr_k":0,"seed":55,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=0"}'),
 ('WAVE-GF-001-P1-k0-s89', 1, 2, 89, 6.180e-4, 'adamw', 256, 'gf16',
  '{"k":0,"phi_lr_k":0,"seed":89,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=0"}');

-- k=1  lr ≈ 4.854e-4
INSERT INTO experiment_queue (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json) VALUES
 ('WAVE-GF-001-P1-k1-s34', 1, 2, 34, 4.854e-4, 'adamw', 256, 'gf16',
  '{"k":1,"phi_lr_k":1,"seed":34,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=1"}'),
 ('WAVE-GF-001-P1-k1-s55', 1, 2, 55, 4.854e-4, 'adamw', 256, 'gf16',
  '{"k":1,"phi_lr_k":1,"seed":55,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=1"}'),
 ('WAVE-GF-001-P1-k1-s89', 1, 2, 89, 4.854e-4, 'adamw', 256, 'gf16',
  '{"k":1,"phi_lr_k":1,"seed":89,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=1"}');

-- k=2  lr ≈ 3.814e-4
INSERT INTO experiment_queue (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json) VALUES
 ('WAVE-GF-001-P1-k2-s34', 1, 3, 34, 3.814e-4, 'adamw', 256, 'gf16',
  '{"k":2,"phi_lr_k":2,"seed":34,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=2"}'),
 ('WAVE-GF-001-P1-k2-s55', 1, 3, 55, 3.814e-4, 'adamw', 256, 'gf16',
  '{"k":2,"phi_lr_k":2,"seed":55,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=2"}'),
 ('WAVE-GF-001-P1-k2-s89', 1, 3, 89, 3.814e-4, 'adamw', 256, 'gf16',
  '{"k":2,"phi_lr_k":2,"seed":89,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=2"}');

-- k=3  lr ≈ 2.998e-4
INSERT INTO experiment_queue (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json) VALUES
 ('WAVE-GF-001-P1-k3-s34', 1, 3, 34, 2.998e-4, 'adamw', 256, 'gf16',
  '{"k":3,"phi_lr_k":3,"seed":34,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=3"}'),
 ('WAVE-GF-001-P1-k3-s55', 1, 3, 55, 2.998e-4, 'adamw', 256, 'gf16',
  '{"k":3,"phi_lr_k":3,"seed":55,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=3"}'),
 ('WAVE-GF-001-P1-k3-s89', 1, 3, 89, 2.998e-4, 'adamw', 256, 'gf16',
  '{"k":3,"phi_lr_k":3,"seed":89,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=3"}');

-- k=4  lr ≈ 2.356e-4
INSERT INTO experiment_queue (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json) VALUES
 ('WAVE-GF-001-P1-k4-s34', 1, 4, 34, 2.356e-4, 'adamw', 256, 'gf16',
  '{"k":4,"phi_lr_k":4,"seed":34,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=4"}'),
 ('WAVE-GF-001-P1-k4-s55', 1, 4, 55, 2.356e-4, 'adamw', 256, 'gf16',
  '{"k":4,"phi_lr_k":4,"seed":55,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=4"}'),
 ('WAVE-GF-001-P1-k4-s89', 1, 4, 89, 2.356e-4, 'adamw', 256, 'gf16',
  '{"k":4,"phi_lr_k":4,"seed":89,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=4"}');

-- k=5  lr ≈ 1.851e-4
INSERT INTO experiment_queue (canon_name, phase, priority, seed, lr, optimizer, d_model, dtype, config_json) VALUES
 ('WAVE-GF-001-P1-k5-s34', 1, 4, 34, 1.851e-4, 'adamw', 256, 'gf16',
  '{"k":5,"phi_lr_k":5,"seed":34,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=5"}'),
 ('WAVE-GF-001-P1-k5-s55', 1, 4, 55, 1.851e-4, 'adamw', 256, 'gf16',
  '{"k":5,"phi_lr_k":5,"seed":55,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=5"}'),
 ('WAVE-GF-001-P1-k5-s89', 1, 4, 89, 1.851e-4, 'adamw', 256, 'gf16',
  '{"k":5,"phi_lr_k":5,"seed":89,"d":256,"batch":64,"steps":27000,"notes":"Quick-3 k=5"}');
