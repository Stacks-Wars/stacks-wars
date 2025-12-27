"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Loader2 } from "lucide-react";

import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import {
	Form,
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import { useUserStore } from "@/lib/stores/user";
import type { User } from "@/lib/definitions";
import { useRouter } from "next/navigation";

// Validation schema
const editProfileSchema = z.object({
	username: z.string().refine(
		(val) => {
			if (!val || val === "") return true;
			return (
				val.length >= 3 &&
				val.length <= 20 &&
				/^[a-zA-Z0-9_]+$/.test(val)
			);
		},
		{
			message:
				"Username must be 3-20 characters and contain only letters, numbers, and underscores",
		}
	),
	displayName: z.string().refine(
		(val) => {
			if (!val || val === "") return true;
			return val.length >= 2 && val.length <= 50;
		},
		{ message: "Display name must be 2-50 characters" }
	),
});

type EditProfileFormValues = z.infer<typeof editProfileSchema>;

interface EditProfileProps {
	children: React.ReactNode;
	currentUser: User;
}

export default function EditProfile({
	children,
	currentUser,
}: EditProfileProps) {
	const [open, setOpen] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const updateUser = useUserStore((state) => state.updateUser);
	const router = useRouter();

	const form = useForm<EditProfileFormValues>({
		resolver: zodResolver(editProfileSchema),
		defaultValues: {
			username: currentUser.username || "",
			displayName: currentUser.displayName || "",
		},
	});

	const onSubmit = async (values: EditProfileFormValues) => {
		setError(null);

		// Only include fields that have changed and are not empty
		const payload: Partial<User> = {};

		if (values.username && values.username !== currentUser.username) {
			payload.username = values.username;
		}

		if (
			values.displayName &&
			values.displayName !== currentUser.displayName
		) {
			payload.displayName = values.displayName;
		}

		// If nothing changed, just close the modal
		if (Object.keys(payload).length === 0) {
			setOpen(false);
			return;
		}

		try {
			const response = await ApiClient.patch<User>(
				"/api/user/profile",
				payload
			);

			if (response.error) {
				setError(response.error);
				return;
			}

			if (response.data) {
				// Update user store
				updateUser(response.data);
				setOpen(false);
				router.refresh();
			}
		} catch (err) {
			const errorMessage =
				err instanceof Error ? err.message : "Failed to update profile";
			setError(errorMessage);
		}
	};

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger asChild>{children}</DialogTrigger>
			<DialogContent className="sm:max-w-106.25 rounded-4xl">
				<DialogHeader>
					<DialogTitle className="text-xl sm:text-2xl">
						Edit Profile
					</DialogTitle>
					<DialogDescription className="text-sm sm:text-base">
						Update your profile.
					</DialogDescription>
				</DialogHeader>

				<Form {...form}>
					<form
						onSubmit={form.handleSubmit(onSubmit)}
						className="space-y-4 sm:space-y-6"
					>
						<FormField
							control={form.control}
							name="username"
							render={({ field }) => (
								<FormItem>
									<FormLabel className="text-sm sm:text-base">
										Username
									</FormLabel>
									<FormControl>
										<Input
											placeholder="Enter username"
											className="text-sm sm:text-base h-10 sm:h-12"
											{...field}
										/>
									</FormControl>
									<FormDescription className="text-xs sm:text-sm">
										3-20 characters.
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						<FormField<EditProfileFormValues>
							control={form.control}
							name="displayName"
							render={({ field }) => (
								<FormItem>
									<FormLabel className="text-sm sm:text-base">
										Display Name
									</FormLabel>
									<FormControl>
										<Input
											placeholder="Enter display name"
											className="text-sm sm:text-base h-10 sm:h-12"
											{...field}
										/>
									</FormControl>
									<FormDescription className="text-xs sm:text-sm">
										2-50 characters.
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>

						{error && (
							<div className="text-sm text-destructive">
								{error}
							</div>
						)}

						<div className="flex justify-end gap-3">
							<Button
								type="button"
								variant="outline"
								onClick={() => setOpen(false)}
								className="rounded-full text-sm sm:text-base"
							>
								Cancel
							</Button>
							<Button
								type="submit"
								className="rounded-full text-sm sm:text-base has-[>svg]:px-8"
								disabled={form.formState.isSubmitting}
							>
								{form.formState.isSubmitting ? (
									<>
										<Loader2 className="mr-2 h-4 w-4 animate-spin" />
										Saving...
									</>
								) : (
									"Save Changes"
								)}
							</Button>
						</div>
					</form>
				</Form>
			</DialogContent>
		</Dialog>
	);
}
