import { User } from "lucide-react";
import { useEffect, useState } from "react";

interface GameRuleProps {
	currentRule: string;
}

export default function GameRule({ currentRule }: GameRuleProps) {
	const [isPageVisible, setIsPageVisible] = useState(true);
	const [isWindowFocused, setIsWindowFocused] = useState(true);

	useEffect(() => {
		// Handle page visibility change (tab switching)
		const handleVisibilityChange = () => {
			setIsPageVisible(!document.hidden);
		};

		// Handle window focus/blur (switching to other apps)
		const handleFocus = () => setIsWindowFocused(true);
		const handleBlur = () => setIsWindowFocused(false);

		// Handle common screenshot key combinations
		const handleKeyDown = (e: KeyboardEvent) => {
			// Detect Print Screen, Alt+Print Screen, Win+Print Screen, etc.
			if (
				e.key === "PrintScreen" ||
				(e.altKey && e.key === "PrintScreen") ||
				(e.metaKey && e.shiftKey && e.key === "S") || // Mac screenshot
				(e.metaKey && e.shiftKey && e.key === "4") || // Mac area screenshot
				(e.ctrlKey && e.shiftKey && e.key === "S") // Some apps
			) {
				setIsPageVisible(false);
				// Briefly hide the rule when screenshot keys are detected
				setTimeout(() => setIsPageVisible(true), 2000);
			}
		};

		document.addEventListener("visibilitychange", handleVisibilityChange);
		window.addEventListener("focus", handleFocus);
		window.addEventListener("blur", handleBlur);
		document.addEventListener("keydown", handleKeyDown);

		return () => {
			document.removeEventListener(
				"visibilitychange",
				handleVisibilityChange
			);
			window.removeEventListener("focus", handleFocus);
			window.removeEventListener("blur", handleBlur);
			document.removeEventListener("keydown", handleKeyDown);
		};
	}, []);

	const shouldHideRule = !isPageVisible || !isWindowFocused;

	return (
		<div className="bg-primary/10 border rounded-xl select-none">
			<div className="flex items-center gap-2 sm:gap-3 p-3 sm:p-4">
				<div className="size-12 rounded-full bg-primary/10 flex items-center justify-center">
					<User className="size-6 text-primary" />
				</div>
				<div>
					<p className="text-base text-primary font-medium select-none">
						Current Rule
					</p>
					<p
						className={`text-sm font-bold select-none unselectable transition-all duration-300 ${
							shouldHideRule ? "blur-lg opacity-30" : ""
						}`}
						onCopy={(e) => {
							e.preventDefault();
						}}
					>
						{shouldHideRule ? "••••••••••••••••••••" : currentRule}
					</p>
					{shouldHideRule && (
						<p className="text-xs text-muted-foreground mt-1">
							Rule hidden - focus on game to continue
						</p>
					)}
				</div>
			</div>
		</div>
	);
}
