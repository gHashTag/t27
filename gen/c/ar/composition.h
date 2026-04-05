/* AUTO-GENERATED from specs/ar/composition.t27 — DO NOT EDIT */
/* Ring: 18 | Module: Composition | ML+AR Composition for CLARA */

#ifndef COMPOSITION_H
#define COMPOSITION_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ═══════════════════════════════════════════════════════════════ */
/* Dependencies — from ar::ternary_logic                          */
/* ═══════════════════════════════════════════════════════════════ */

typedef int8_t Trit;

#define K_FALSE   ((Trit)-1)
#define K_UNKNOWN ((Trit)0)
#define K_TRUE    ((Trit)1)

/* GF16 = u16 (half-precision confidence) */
typedef uint16_t GF16;

/* ═══════════════════════════════════════════════════════════════ */
/* Constants                                                       */
/* ═══════════════════════════════════════════════════════════════ */

#define MAX_COMPOSITION_DEPTH  5
#define GF16_ONE               ((GF16)0x3C00)
#define GF16_ZERO              ((GF16)0x0000)
#define MAX_STEPS              10

/* ═══════════════════════════════════════════════════════════════ */
/* Types                                                           */
/* ═══════════════════════════════════════════════════════════════ */

/* Composition pattern — how ML and AR components combine */
typedef enum {
    PATTERN_CNN_RULES       = 0,
    PATTERN_MLP_BAYESIAN    = 1,
    PATTERN_TRANSFORMER_XAI = 2,
    PATTERN_RL_GUARDRAILS   = 3
} CompositionPattern;

/* Explanation format style */
typedef enum {
    STYLE_NATURAL = 0,
    STYLE_FITCH   = 1,
    STYLE_COMPACT = 2
} FormatStyle;

/* Horn clause for AR inference */
typedef struct {
    Trit antecedent;
    Trit consequent;
} HornClause;

/* Derivation step within a proof trace */
typedef struct {
    uint8_t  step_number;
    Trit     output_fact;
    GF16     confidence;
    Trit     k3_value;
} DerivationStep;

/* Proof trace: bounded derivation chain */
typedef struct {
    DerivationStep steps[MAX_STEPS];
    uint8_t        step_count;
} ProofTrace;

/* Explanation structure */
typedef struct {
    ProofTrace  trace;
    FormatStyle style;
    GF16        confidence;
    uint8_t     step_count;
} Explanation;

/* ML component abstraction (pluggable) */
typedef struct {
    const char *component_type;
    uint32_t    input_shape[4];
    uint32_t    output_shape[2];
    GF16        confidence;
} MLComponent;

/* AR component abstraction */
typedef struct {
    const HornClause *rule_set;
    size_t            rule_count;
    FormatStyle       explanation_style;
    GF16              confidence_threshold;
} ARComponent;

/* Composed pipeline — ML + AR combination */
typedef struct {
    CompositionPattern pattern;
    MLComponent        ml_component;
    ARComponent        ar_component;
    GF16               pipeline_confidence;
    uint8_t            steps_completed;
    Explanation        explanation;
    bool               terminated;
} ComposedPipeline;

/* Composition result */
typedef struct {
    Trit        prediction;
    GF16        confidence;
    Explanation explanation;
    ProofTrace  proof_trace;
    GF16        satisfaction;
} CompositionResult;

/* ═══════════════════════════════════════════════════════════════ */
/* Function declarations                                           */
/* ═══════════════════════════════════════════════════════════════ */

/* GF16 helpers */
GF16  gf16_from_f32(float value);
float gf16_to_f32(GF16 gf);

/* Core functions */
Trit  f32_to_trit(float value);
GF16  combine_confidence(GF16 ml_conf, GF16 ar_conf);
GF16  calculate_satisfaction(const ComposedPipeline *pipeline,
                             Trit prediction, GF16 confidence);
Trit  evaluate_ar_rules(const ARComponent *ar_comp, Trit input_decision);
ComposedPipeline create_pipeline(CompositionPattern pattern,
                                  GF16 ml_confidence,
                                  const HornClause *rules,
                                  size_t rule_count);

/* Pattern-specific composition */
CompositionResult compose_cnn_rules(const float *cnn_features,
                                     size_t feature_count,
                                     GF16 cnn_confidence,
                                     const HornClause *rules,
                                     size_t rule_count,
                                     GF16 confidence_threshold);

CompositionResult compose_mlp_bayesian(const float *mlp_input,
                                        size_t input_count,
                                        GF16 mlp_confidence,
                                        float bayesian_prior,
                                        GF16 confidence_threshold);

CompositionResult compose_transformer_xai(const float *transformer_input,
                                           size_t input_count,
                                           GF16 transformer_confidence,
                                           uint8_t max_steps,
                                           FormatStyle style);

CompositionResult compose_rl_guardrails(const float *state,
                                         size_t state_count,
                                         GF16 policy_confidence,
                                         const HornClause *guardrails,
                                         size_t guardrail_count,
                                         GF16 confidence_threshold);

/* Generic dispatch */
CompositionResult compose(CompositionPattern pattern,
                          const float *ml_input,
                          size_t input_count,
                          GF16 ml_confidence,
                          const HornClause *ar_rules,
                          size_t ar_rule_count,
                          GF16 confidence_threshold);

#endif /* COMPOSITION_H */
