import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { useState } from "react";
import { useRouter } from "next/navigation";

interface BackProps {
	isOut: boolean;
	isSpectator: boolean;
	disconnect: () => void;
	disconnectChat: () => void;
}

export default function Back({
	isOut,
	disconnect,
	disconnectChat,
	isSpectator,
}: BackProps) {
	const [open, setOpen] = useState(false);
	const router = useRouter();

	return (
		<>
			{isOut || isSpectator ? (
				<Button
					variant={"link"}
					className="inline-flex items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors mb-4 sm:mb-6"
					onClick={() => {
						disconnect();
						disconnectChat();
						router.back();
					}}
				>
					<ArrowLeft className="h-4 w-4" />
					Back
				</Button>
			) : (
				<Dialog open={open} onOpenChange={setOpen}>
					<DialogTrigger asChild>
						<Button
							variant={"link"}
							className="inline-flex items-center gap-1 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors mb-4 sm:mb-6"
						>
							<ArrowLeft className="h-4 w-4" />
							<span>Quit game</span>
						</Button>
					</DialogTrigger>
					<DialogContent className="bg-gradient-to-b from-primary/90 to-primary/70 border-0 text-white">
						<DialogHeader>
							<DialogTitle>
								Are you sure you want to quit?
							</DialogTitle>
							<DialogDescription>
								You are going to lose game progress if you quit.
								This action cannot be undone.
							</DialogDescription>
						</DialogHeader>
						<DialogFooter>
							<Button
								variant="outline"
								onClick={() => setOpen(false)}
							>
								Cancel
							</Button>
							<Button
								variant="destructive"
								onClick={() => {
									setOpen(false);
									disconnect();
									disconnectChat();
									router.back();
								}}
							>
								Quit Game
							</Button>
						</DialogFooter>
					</DialogContent>
				</Dialog>
			)}
		</>
	);
}
