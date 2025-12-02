import { Badge } from "@/components/ui/badge";
import { MdOutlineTimer } from "react-icons/md";
import { cn } from "@/lib/utils";
export default function GameTimer({ timeLeft }: { timeLeft: number }) {
	return (
		<div
			className={`${cn(
				timeLeft <= 5
					? "bg-destructive/50 border-destructive/90"
					: "bg-primary/10",
				"border rounded-xl w-full flex items-center justify-center"
			)}`}
		>
			<div className="flex items-center justify-between p-3 sm:p-4 gap-2 w-full">
				<p className="text-base font-medium">Time Left</p>
				<Badge
					variant={timeLeft <= 5 ? "destructive" : "default"}
					className="text-base font-bold text-foreground px-3 sm:px-4 py-1 sm:py-2"
				>
					<MdOutlineTimer className="size-4 mr-1" />
					{timeLeft}s
				</Badge>
			</div>
		</div>
	);
}
