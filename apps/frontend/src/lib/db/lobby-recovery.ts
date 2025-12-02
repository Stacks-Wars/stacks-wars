import Dexie, { Table } from "dexie";

export interface LobbyRecoveryData {
	id: string; // user wallet address + timestamp
	userAddress: string;
	gameId: string;
	formData: {
		name: string;
		description?: string;
		withPool: boolean;
		amount?: number;
	};
	deployedContract?: {
		contractName: string;
		contractAddress: `${string}.${string}`;
		entryAmount: number;
		txId: string;
	};
	joinedContract?: {
		contractAddress: `${string}.${string}`;
		txId: string;
		entryAmount: number;
	};
	createdAt: number;
	updatedAt: number;
	status: "pending" | "deployed" | "joined";
}

export interface SponsoredLobbyRecoveryData {
	id: string; // user wallet address + timestamp
	userAddress: string;
	gameId: string;
	formData: {
		name: string;
		description?: string;
		token: string;
		poolSize: number;
	};
	deployedContract?: {
		contractName: string;
		contractAddress: `${string}.${string}`;
		poolSize: number;
		txId: string;
		token: string;
	};
	joinedContract?: {
		contractAddress: `${string}.${string}`;
		txId: string;
		poolSize: number;
		token: string;
	};
	createdAt: number;
	updatedAt: number;
	status: "pending" | "deployed" | "joined";
}

export class LobbyRecoveryDB extends Dexie {
	lobbyRecovery!: Table<LobbyRecoveryData>;
	sponsoredLobbyRecovery!: Table<SponsoredLobbyRecoveryData>;

	constructor() {
		super("LobbyRecoveryDB");
		this.version(1).stores({
			lobbyRecovery: "id, userAddress, gameId, status, createdAt",
			sponsoredLobbyRecovery:
				"id, userAddress, gameId, status, createdAt",
		});
	}
}

export const lobbyRecoveryDB = new LobbyRecoveryDB();

// Helper functions for regular lobby recovery
export const saveLobbyRecoveryData = async (
	data: Omit<LobbyRecoveryData, "id" | "createdAt" | "updatedAt">
) => {
	const id = `${data.userAddress}-${data.gameId}-${Date.now()}`;
	const now = Date.now();

	const recoveryData: LobbyRecoveryData = {
		...data,
		id,
		createdAt: now,
		updatedAt: now,
	};

	await lobbyRecoveryDB.lobbyRecovery.put(recoveryData);
	return id;
};

export const updateLobbyRecoveryData = async (
	id: string,
	updates: Partial<LobbyRecoveryData>
) => {
	await lobbyRecoveryDB.lobbyRecovery.update(id, {
		...updates,
		updatedAt: Date.now(),
	});
};

export const getLobbyRecoveryData = async (
	userAddress: string,
	gameId: string
): Promise<LobbyRecoveryData | undefined> => {
	const recoveryData = await lobbyRecoveryDB.lobbyRecovery
		.where("userAddress")
		.equals(userAddress)
		.and((item) => item.gameId === gameId)
		.reverse()
		.first();

	return recoveryData;
};

export const deleteLobbyRecoveryData = async (id: string) => {
	await lobbyRecoveryDB.lobbyRecovery.delete(id);
};

// Helper functions for sponsored lobby recovery
export const saveSponsoredLobbyRecoveryData = async (
	data: Omit<SponsoredLobbyRecoveryData, "id" | "createdAt" | "updatedAt">
) => {
	const id = `${data.userAddress}-${data.gameId}-sponsored-${Date.now()}`;
	const now = Date.now();

	const recoveryData: SponsoredLobbyRecoveryData = {
		...data,
		id,
		createdAt: now,
		updatedAt: now,
	};

	await lobbyRecoveryDB.sponsoredLobbyRecovery.put(recoveryData);
	return id;
};

export const updateSponsoredLobbyRecoveryData = async (
	id: string,
	updates: Partial<SponsoredLobbyRecoveryData>
) => {
	await lobbyRecoveryDB.sponsoredLobbyRecovery.update(id, {
		...updates,
		updatedAt: Date.now(),
	});
};

export const getSponsoredLobbyRecoveryData = async (
	userAddress: string,
	gameId: string
): Promise<SponsoredLobbyRecoveryData | undefined> => {
	const recoveryData = await lobbyRecoveryDB.sponsoredLobbyRecovery
		.where("userAddress")
		.equals(userAddress)
		.and((item) => item.gameId === gameId)
		.reverse()
		.first();

	return recoveryData;
};

export const deleteSponsoredLobbyRecoveryData = async (id: string) => {
	await lobbyRecoveryDB.sponsoredLobbyRecovery.delete(id);
};

// Clean up old recovery data (older than 24 hours)
export const cleanupOldRecoveryData = async () => {
	const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000;

	await lobbyRecoveryDB.lobbyRecovery
		.where("createdAt")
		.below(oneDayAgo)
		.delete();

	await lobbyRecoveryDB.sponsoredLobbyRecovery
		.where("createdAt")
		.below(oneDayAgo)
		.delete();
};
