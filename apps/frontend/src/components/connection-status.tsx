import { RefreshCw, Wifi, WifiOff, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ConnectionStatusProps {
	readyState: WebSocket["readyState"];
	latency: number | null;
	className?: string;
	onReconnect?: () => void;
	reconnecting?: boolean;
}

export default function ConnectionStatus({
	readyState,
	latency,
	className,
	onReconnect,
	reconnecting = false,
}: ConnectionStatusProps) {
	const getLatencyColor = (ms: number) => {
		if (ms <= 60) return "text-green-500"; // very good
		if (ms <= 120) return "text-yellow-500"; // good
		if (ms <= 250) return "text-orange-500"; // bad
		return "text-red-500"; // very bad
	};

	const getConnectionIcon = () => {
		if (readyState === WebSocket.OPEN) {
			return <Wifi className="h-3 w-3" />;
		} else if (readyState === WebSocket.CONNECTING || reconnecting) {
			return <RefreshCw className="h-3 w-3 animate-spin" />;
		} else {
			return <WifiOff className="h-3 w-3" />;
		}
	};

	const getConnectionStatus = () => {
		if (readyState === WebSocket.CONNECTING || reconnecting) {
			return (
				<div
					className={`flex items-center gap-2 text-xs text-blue-500 ${className}`}
				>
					{getConnectionIcon()}
					<span>connecting...</span>
				</div>
			);
		}

		if (readyState === WebSocket.CLOSED && !reconnecting) {
			return (
				<div
					className={`flex items-center gap-2 text-xs text-red-500 ${className}`}
				>
					{getConnectionIcon()}
					<span>disconnected</span>
					{onReconnect && (
						<Button
							variant="ghost"
							size="sm"
							className="h-5 px-2 text-xs text-red-500 hover:bg-red-50 hover:text-red-600"
							onClick={onReconnect}
						>
							<RefreshCw className="mr-1 h-3 w-3" />
							retry
						</Button>
					)}
				</div>
			);
		}

		if (readyState === WebSocket.CLOSING) {
			return (
				<div
					className={`flex items-center gap-2 text-xs text-orange-500 ${className}`}
				>
					<AlertCircle className="h-3 w-3" />
					<span>closing...</span>
				</div>
			);
		}

		if (readyState === WebSocket.OPEN) {
			if (latency !== null) {
				return (
					<div
						className={`flex items-center gap-2 text-xs ${getLatencyColor(latency)} ${className}`}
					>
						{getConnectionIcon()}
						<span>{Math.min(latency, 999)}ms</span>
					</div>
				);
			} else {
				return (
					<div
						className={`flex items-center gap-2 text-xs text-green-500 ${className}`}
					>
						{getConnectionIcon()}
						<span>connecting...</span>
					</div>
				);
			}
		}

		return null;
	};

	return getConnectionStatus();
}
