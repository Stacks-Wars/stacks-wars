"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { User } from "@/types/schema/user";
import { apiRequest } from "@/lib/api";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { Edit, Save, X } from "lucide-react";

const RESERVED_USERNAMES = [
	"game",
	"games",
	"lobby",
	"leaderboard",
	"lexi-wars",
	"profile",
	"user",
	"admin",
	"api",
	"auth",
	"login",
	"signup",
	"settings",
	"dashboard",
];

interface ProfileEditProps {
	user: User;
}

export default function ProfileEdit({ user }: ProfileEditProps) {
	const [isEditingDisplayName, setIsEditingDisplayName] = useState(false);
	const [username, setUsername] = useState(user.username || "");
	const [displayName, setDisplayName] = useState(user.displayName || "");
	const [loading, setLoading] = useState(false);
	const router = useRouter();

	// Username validation function
	const validateUsername = (username: string): string | null => {
		const trimmed = username.trim().toLowerCase();

		if (!trimmed) {
			return "Username cannot be empty";
		}

		if (trimmed.length < 3) {
			return "Username must be at least 3 characters long";
		}

		if (trimmed.length > 20) {
			return "Username must be 20 characters or less";
		}

		// Check for reserved usernames
		if (RESERVED_USERNAMES.includes(trimmed)) {
			return "This username is reserved and cannot be used";
		}

		// Check for valid characters (alphanumeric, underscore, hyphen)
		if (!/^[a-zA-Z0-9_-]+$/.test(trimmed)) {
			return "Username can only contain letters, numbers, underscores, and hyphens";
		}

		// Cannot start or end with special characters
		if (/^[-_]|[-_]$/.test(trimmed)) {
			return "Username cannot start or end with underscore or hyphen";
		}

		return null; // Valid username
	};

	const handleUpdateUsername = async () => {
		const validationError = validateUsername(username);
		if (validationError) {
			toast.error(validationError);
			return;
		}

		setLoading(true);
		try {
			await apiRequest({
				path: "/user/username",
				method: "PATCH",
				body: { username: username.trim() },
			});

			toast.success("Username updated successfully!");
			router.refresh();
		} catch (error) {
			console.error("Failed to update username:", error);
			toast.error("Failed to update username. Please try again.");
		} finally {
			setLoading(false);
		}
	};

	const handleUpdateDisplayName = async () => {
		setLoading(true);
		try {
			await apiRequest({
				path: "/user/display_name",
				method: "PATCH",
				body: { display_name: displayName.trim() || null },
			});

			toast.success("Display name updated successfully!");
			setIsEditingDisplayName(false);
			router.refresh();
		} catch (error) {
			console.error("Failed to update display name:", error);
			toast.error("Failed to update display name. Please try again.");
		} finally {
			setLoading(false);
		}
	};

	const cancelEditDisplayName = () => {
		setDisplayName(user.displayName || "");
		setIsEditingDisplayName(false);
	};

	// Check if current username input is valid
	const isUsernameValid = !validateUsername(username);

	return (
		<Card>
			<CardHeader>
				<CardTitle className="flex items-center gap-2">
					<Edit className="h-5 w-5" />
					Edit Profile
				</CardTitle>
			</CardHeader>
			<CardContent className="space-y-6">
				{/* Username Section */}
				{!user.username ? (
					<div className="space-y-2">
						<Label htmlFor="username">Username</Label>
						<div className="flex gap-2">
							<Input
								id="username"
								value={username}
								onChange={(e) => setUsername(e.target.value)}
								placeholder="Add a username"
								disabled={loading}
								className={
									username && !isUsernameValid
										? "border-destructive focus-visible:ring-destructive"
										: ""
								}
							/>
							<Button
								size="sm"
								onClick={handleUpdateUsername}
								disabled={
									loading ||
									!username.trim() ||
									!isUsernameValid
								}
							>
								<Save className="h-4 w-4" />
							</Button>
						</div>

						{/* Show validation error if any */}
						{username && !isUsernameValid && (
							<p className="text-sm text-destructive">
								{validateUsername(username)}
							</p>
						)}

						<div className="space-y-1">
							<p className="text-sm text-muted-foreground">
								Add a username to make your profile easier to
								find
							</p>
							<p className="text-sm text-muted-foreground">
								Username cannot be changed once set
							</p>
							<p className="text-xs text-muted-foreground">
								3-20 characters, letters, numbers, underscore,
								and hyphen only
							</p>
						</div>
					</div>
				) : (
					<div className="space-y-2">
						<Label>Username</Label>
						<Input value={`@${user.username}`} disabled />
						<p className="text-sm text-muted-foreground">
							Username cannot be changed once set
						</p>
					</div>
				)}

				{/* Display Name Section - remains the same */}
				<div className="space-y-2">
					<Label htmlFor="displayName">Display Name</Label>
					{isEditingDisplayName ? (
						<div className="flex gap-2">
							<Input
								id="displayName"
								value={displayName}
								onChange={(e) => setDisplayName(e.target.value)}
								placeholder="Enter display name"
								disabled={loading}
							/>
							<Button
								size="sm"
								onClick={handleUpdateDisplayName}
								disabled={loading}
							>
								<Save className="h-4 w-4" />
							</Button>
							<Button
								size="sm"
								variant="outline"
								onClick={cancelEditDisplayName}
								disabled={loading}
							>
								<X className="h-4 w-4" />
							</Button>
						</div>
					) : (
						<div className="flex gap-2">
							<Input
								value={displayName || ""}
								placeholder={
									user.displayName
										? user.displayName
										: "Add a display name"
								}
								disabled
							/>
							<Button
								size="sm"
								variant="outline"
								onClick={() => setIsEditingDisplayName(true)}
							>
								<Edit className="h-4 w-4" />
							</Button>
						</div>
					)}
					<p className="text-sm text-muted-foreground">
						Your display name is shown to other players
					</p>
				</div>
			</CardContent>
		</Card>
	);
}
