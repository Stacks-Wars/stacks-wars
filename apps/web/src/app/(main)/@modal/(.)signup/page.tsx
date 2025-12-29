"use client";
import { useRouter } from "next/navigation";
import { AuthDialog } from "../_components/auth-dialog";

export default function SignupModal() {
	const router = useRouter();

	return (
		<AuthDialog
			mode="signup"
			open={true}
			onOpenChange={() => router.back()}
		/>
	);
}
