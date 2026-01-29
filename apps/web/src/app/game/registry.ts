/**
 * Game Plugin Registry
 *
 * Central registry of all available game plugins.
 * Add new games here to make them available to the engine.
 */

import type { GamePlugin, PluginRegistry } from "@/lib/definitions";
import { LexiWarsPlugin } from "./lexi-wars/plugin";

// Registry maps game path to plugin
export const gamePlugins: PluginRegistry = {
	[LexiWarsPlugin.path]: LexiWarsPlugin as GamePlugin,
};

/**
 * Get a game plugin by its path
 */
export function getGamePlugin(gamePath: string): GamePlugin | undefined {
	return gamePlugins[gamePath];
}

/**
 * Get all registered game plugins
 */
export function getAllGamePlugins(): GamePlugin[] {
	return Object.values(gamePlugins);
}

/**
 * Check if a game plugin exists
 */
export function hasGamePlugin(gamePath: string): boolean {
	return gamePath in gamePlugins;
}
