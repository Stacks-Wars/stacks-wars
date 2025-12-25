const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

export interface ApiResponse<T> {
	data?: T;
	error?: string;
	status: number;
}

export class ApiClient {
	private static getHeaders(): HeadersInit {
		return {
			"Content-Type": "application/json",
			Accept: "application/json",
		};
	}

	static async get<T>(endpoint: string): Promise<ApiResponse<T>> {
		try {
			const response = await fetch(`${API_BASE_URL}${endpoint}`, {
				method: "GET",
				headers: this.getHeaders(),
				credentials: "include",
			});

			const data = await response.json();

			if (!response.ok) {
				return {
					error: data.message || "Request failed",
					status: response.status,
				};
			}

			return {
				data,
				status: response.status,
			};
		} catch (error) {
			return {
				error: error instanceof Error ? error.message : "Network error",
				status: 500,
			};
		}
	}

	static async post<T>(
		endpoint: string,
		body?: any
	): Promise<ApiResponse<T>> {
		try {
			const response = await fetch(`${API_BASE_URL}${endpoint}`, {
				method: "POST",
				headers: this.getHeaders(),
				credentials: "include",
				body: body ? JSON.stringify(body) : undefined,
			});

			const data = await response.json();

			if (!response.ok) {
				return {
					error: data.message || "Request failed",
					status: response.status,
				};
			}

			return {
				data,
				status: response.status,
			};
		} catch (error) {
			return {
				error: error instanceof Error ? error.message : "Network error",
				status: 500,
			};
		}
	}

	static async put<T>(endpoint: string, body?: any): Promise<ApiResponse<T>> {
		try {
			const response = await fetch(`${API_BASE_URL}${endpoint}`, {
				method: "PUT",
				headers: this.getHeaders(),
				credentials: "include",
				body: body ? JSON.stringify(body) : undefined,
			});

			const data = await response.json();

			if (!response.ok) {
				return {
					error: data.message || "Request failed",
					status: response.status,
				};
			}

			return {
				data,
				status: response.status,
			};
		} catch (error) {
			return {
				error: error instanceof Error ? error.message : "Network error",
				status: 500,
			};
		}
	}

	static async delete<T>(endpoint: string): Promise<ApiResponse<T>> {
		try {
			const response = await fetch(`${API_BASE_URL}${endpoint}`, {
				method: "DELETE",
				headers: this.getHeaders(),
				credentials: "include",
			});

			const data = await response.json();

			if (!response.ok) {
				return {
					error: data.message || "Request failed",
					status: response.status,
				};
			}

			return {
				data,
				status: response.status,
			};
		} catch (error) {
			return {
				error: error instanceof Error ? error.message : "Network error",
				status: 500,
			};
		}
	}
}
