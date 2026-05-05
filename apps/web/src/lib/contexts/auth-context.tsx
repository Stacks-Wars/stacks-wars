"use client";

import { createContext, useContext, useEffect, type ReactNode } from "react";
import { ApiClient } from "@/lib/api/client";
import { useUserActions } from "@/lib/stores/user";
import type { User } from "@/lib/definitions";

interface AuthContextValue {
	refreshAuth: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
	const { setUser, clearUser, setLoading } = useUserActions();

	const checkAuth = async () => {
		setLoading(true);
		try {
			const response = await ApiClient.get<User>("/api/me");

			if (response.status === 200 && response.data) {
				setUser(response.data);
			} else {
				clearUser();
			}
		} catch (error) {
			console.error("Failed to check authentication:", error);
			clearUser();
		}
	};

	// Check auth on mount
	useEffect(() => {
		checkAuth();
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	return (
		<AuthContext.Provider value={{ refreshAuth: checkAuth }}>
			{children}
		</AuthContext.Provider>
	);
}

export function useAuth() {
	const context = useContext(AuthContext);
	if (!context) {
		throw new Error("useAuth must be used within AuthProvider");
	}
	return context;
}
