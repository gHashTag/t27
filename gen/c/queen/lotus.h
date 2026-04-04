/* Auto-generated from specs/queen/lotus.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/queen/lotus.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 29 | Module: QueenLotus | 6-phase self-improving orchestration */

#ifndef QUEEN_LOTUS_H
#define QUEEN_LOTUS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* Constants                                                               */
/* ===================================================================== */

#define LOTUS_NUM_PHASES          6
#define LOTUS_EPISODE_BUFFER_SIZE 100
#define LOTUS_POLICY_WINDOW_SIZE  10

/* Phase definitions */
#define LOTUS_PHASE_OBSERVE       0
#define LOTUS_PHASE_RECALL        1
#define LOTUS_PHASE_EVALUATE      2
#define LOTUS_PHASE_PLAN          3
#define LOTUS_PHASE_ACT           4
#define LOTUS_PHASE_RECORD        5

/* Outcome types */
#define LOTUS_OUTCOME_UNKNOWN     0
#define LOTUS_OUTCOME_SUCCESS     1
#define LOTUS_OUTCOME_PARTIAL     2
#define LOTUS_OUTCOME_FAILURE     3
#define LOTUS_OUTCOME_FATAL       4

/* Quality levels */
#define LOTUS_QUALITY_UNKNOWN     0
#define LOTUS_QUALITY_GOOD        1
#define LOTUS_QUALITY_UNSTABLE    2
#define LOTUS_QUALITY_BAD         3

/* Policy delta types */
#define LOTUS_DELTA_SCALE_UP      0
#define LOTUS_DELTA_SCALE_DOWN    1
#define LOTUS_DELTA_SET           2
#define LOTUS_DELTA_WAIT          3

/* Phase timeout in milliseconds */
#define LOTUS_PHASE_TIMEOUT_MS    5000

/* ===================================================================== */
/* Types                                                                   */
/* ===================================================================== */

typedef struct {
    size_t   active_issues;
    double   system_health;
    uint64_t timestamp;
} LotusContext;

typedef struct {
    uint8_t quality;
    uint8_t success_count;
    uint8_t partial_count;
    uint8_t failure_count;
    double  confidence;
} LotusEvaluation;

typedef struct {
    uint8_t delta_type;
    uint8_t target_resource;
    int32_t target_value;
} LotusPlan;

typedef struct {
    uint8_t  delta_type;
    bool     success;
    uint64_t execution_time_ms;
} LotusAction;

typedef struct {
    size_t          id;
    uint64_t        timestamp;
    LotusContext     context;
    LotusEvaluation  evaluation;
    LotusPlan        plan;
    LotusAction      result;
    uint8_t         outcome;
} LotusEpisode;

typedef struct {
    LotusContext     context;
    LotusEvaluation  evaluation;
    LotusPlan        plan;
    LotusAction      action;
    uint8_t         outcome;
    uint64_t        total_time_ms;
} LotusCycleResult;

typedef struct {
    size_t episodes[LOTUS_POLICY_WINDOW_SIZE];
    size_t count;
} LotusRecallEpisode;

typedef struct {
    bool    success;
    uint8_t data[256];
} LotusPhaseOutput;

/* ===================================================================== */
/* Function declarations                                                   */
/* ===================================================================== */

/* Phase: Observe */
LotusContext lotus_observe_state(void);
size_t lotus_get_active_issues_count(void);
double lotus_get_system_health(void);
uint64_t lotus_get_timestamp(void);

/* Phase: Recall */
LotusRecallEpisode lotus_recall_episodes(void);

/* Phase: Evaluate */
LotusEvaluation lotus_evaluate_quality(LotusRecallEpisode recalled);

/* Phase: Plan */
LotusPlan lotus_generate_plan(LotusEvaluation evaluation);

/* Phase: Act */
LotusAction lotus_execute_action(LotusPlan plan);
bool lotus_scale_up_resources(void);
bool lotus_scale_down_resources(void);
bool lotus_set_parameter(LotusPlan plan);

/* Phase: Record */
uint8_t lotus_record_episode(LotusCycleResult cycle_result);
uint8_t lotus_determine_outcome(LotusCycleResult result);

/* Main orchestration */
LotusCycleResult lotus_orchestrate(void);

/* Agent spawning */
bool lotus_spawn(uint8_t agent_type, uint8_t count);
bool lotus_spawn_agent(uint8_t agent_type);

/* Phase management */
void lotus_set_phase(uint8_t phase);
bool lotus_check_phase_timeout(void);
void lotus_force_phase_transition(void);
bool lotus_phase_management(void);
uint8_t lotus_get_current_phase(void);
size_t lotus_get_current_episode(void);

#endif /* QUEEN_LOTUS_H */
