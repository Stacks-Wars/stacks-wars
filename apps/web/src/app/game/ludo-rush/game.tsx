"use client";

/**
 * Ludo Rush Game Component
 *
 * Reuses the same board UI as classic Ludo, but passes Ludo Rush's
 * reduced safe squares (only 4 entry points) so the board renders correctly.
 */

import type { GamePluginProps } from "@/lib/definitions";
import type { LudoRushState } from "./types";
import { SAFE_SQUARES } from "./types";
import LudoGame from "../ludo/game";

export default function LudoRushGame(props: GamePluginProps<LudoRushState>) {
	return <LudoGame {...props} safeSquares={SAFE_SQUARES} />;
}
