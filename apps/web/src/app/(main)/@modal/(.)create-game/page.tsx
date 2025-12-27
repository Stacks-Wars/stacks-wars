"use client";

import { useRouter } from "next/navigation";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import CreateGameForm from "../../create-game/_components/create-game-form";

export default function CreateGameModal() {
	const router = useRouter();

	const handleClose = () => {
		router.back();
	};

	return (
		<Dialog open={true} onOpenChange={handleClose}>
			<DialogContent className="sm:max-w-2xl rounded-4xl max-h-[90vh] overflow-y-auto">
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
