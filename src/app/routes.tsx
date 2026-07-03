import { Routes, Route, Navigate } from "react-router-dom";
import TranslatePage from "../features/translate/TranslatePage";
import ProviderPage from "../features/provider/ProviderPage";
import ParamsPage from "../features/params/ParamsPage";
import TasksPage from "../features/tasks/TasksPage";
import SettingsPage from "../features/settings/SettingsPage";

export default function AppRoutes() {
  return (
    <Routes>
      <Route path="/translate" element={<TranslatePage />} />
      <Route path="/provider" element={<ProviderPage />} />
      <Route path="/params" element={<ParamsPage />} />
      <Route path="/tasks" element={<TasksPage />} />
      <Route path="/settings" element={<SettingsPage />} />
      <Route path="*" element={<Navigate to="/translate" replace />} />
    </Routes>
  );
}
