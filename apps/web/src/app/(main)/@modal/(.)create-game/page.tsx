"use client";

import { useRouter } from "next/navigation";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import CreateGameForm from "@/components/main/create-game-form";

export default function CreateGameModal() {
	const router = useRouter();

	const handleClose = () => {
		router.back();
	};

	return (
		<Dialog open={true} onOpenChange={handleClose}>
			<DialogContent className="max-h-[90vh] overflow-y-auto rounded-4xl sm:max-w-2xl">
				<DialogHeader>
					<DialogTitle className="text-xl sm:text-2xl">
						Create New Game
					</DialogTitle>
					<DialogDescription className="text-sm sm:text-base">
						Add a new game type to the platform
					</DialogDescription>
				</DialogHeader>
				<CreateGameForm onSuccess={handleClose} />
			</DialogContent>
		</Dialog>
	);
}
