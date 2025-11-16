export function Header({ account, onConnect, onDisconnect }: any) {
  return (
    <header className="bg-black/50 backdrop-blur border-b border-slate-700">
      <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gradient-to-br from-blue-400 to-purple-600 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold">P</span>
          </div>
          <h1 className="text-2xl font-bold text-white">PortableID</h1>
        </div>

        {account ? (
          <div className="flex items-center gap-4">
            <div className="bg-slate-700/50 px-4 py-2 rounded-lg border border-slate-600">
              <span className="text-sm text-slate-300">{account}</span>
            </div>
            <button
              onClick={onDisconnect}
              className="px-4 py-2 bg-red-600/20 hover:bg-red-600/30 text-red-400 rounded-lg transition"
            >
              Disconnect
            </button>
          </div>
        ) : (
          <button
            onClick={onConnect}
            className="px-6 py-2 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg font-medium transition"
          >
            Connect Wallet
          </button>
        )}
      </div>
    </header>
  );
}
