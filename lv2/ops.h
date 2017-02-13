/* All the transformations the Boucle delay engine can perform on the play
 * head.
 *
 * These are also defined in boucle.ttl.
 */
#ifndef __OPS_H__
#define __OPS_H__

#include <lv2/lv2plug.in/ns/lv2core/lv2.h>

typedef enum {
	OP_TYPE_NONE = 0,
	OP_TYPE_REVERSE = 1,
	OP_TYPE_ABSOLUTE_JUMP = 2,
	OP_TYPE_RELATIVE_JUMP = 3,
	OP_TYPE_LOOP_IN_LOOP = 4,
	OP_TYPE_SPEED_RAMP = 5
} OpType;

typedef struct {
} ReverseOp;

typedef struct {
	uint32_t absolute_position;  /* in samples */
} AbsoluteJumpOp;

typedef struct {
	uint32_t relative_position;  /* in samples */
} RelativeJumpOp;

typedef struct {
	uint32_t loop_size;  /* in samples */
} LoopInLoopOp;

typedef struct {
	float start_speed;  /* coefficient */
	float end_speed;  /* coefficient */
} SpeedRampOp;

typedef struct {
	OpType type;
	uint32_t start;  /* in samples */
	uint32_t duration;  /* in samples */
	union {
		ReverseOp reverse;
		AbsoluteJumpOp absolute_jump;
		RelativeJumpOp relative_jump;
		LoopInLoopOp loop_in_loop;
		SpeedRampOp speed_ramp;
	};
} Op;

#endif
