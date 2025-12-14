/**
 * Game Plugin Registry
 *
 * Central registry of all available game plugins.
 * Add new games here to make them available to the engine.
 */

import { PluginRegistry, GamePlugin } from "@/lib/definitions";
import { CoinFlipPlugin } from "./coin-flip/plugin";

export const gamePlugins: PluginRegistry = {
	[CoinFlipPlugin.id]: CoinFlipPlugin as GamePlugin,
};

/**
 * Get a game plugin by its ID
 */
export function getGamePlugin(gameId: string) {
	return gamePlugins[gameId];
}

/**
 * Get all registered game plugins
 */
export function getAllGamePlugins() {
	return Object.values(gamePlugins);
}

/**
 * Check if a game plugin exists
 */
export function hasGamePlugin(gameId: string) {
	return gameId in gamePlugins;
}
