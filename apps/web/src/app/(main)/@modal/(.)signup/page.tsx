"use client";
import { AuthDialog } from "../_components/auth-dialog";

export default function SignupModal() {
	return <AuthDialog mode="signup" open={true} />;
}
