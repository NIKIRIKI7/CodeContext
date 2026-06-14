import pytest
from src.services.strategies.import_strategies import MonorepoImportStrategy
from src.services.import_resolution_service import ImportResolutionService


class TestMonorepoImportStrategy:
    """Тесты стратегии разрешения импортов для монорепозиториев."""

    def setup_method(self):
        self.strategy = MonorepoImportStrategy()

    def test_scoped_package_entry_point(self):
        """
        import { Button } from '@myorg/shared-ui'
        → packages/shared-ui/src/index.ts
        """
        p_no_ext = "/project/packages/shared-ui/src/index"
        parts = ["myorg", "shared-ui"]
        assert self.strategy.is_match(p_no_ext, "myorg/shared-ui", "/myorg/shared-ui", parts) is True

    def test_scoped_package_subpath(self):
        """
        import { format } from '@myorg/core/utils/date'
        → packages/core/src/utils/date.ts
        """
        p_no_ext = "/project/packages/core/src/utils/date"
        parts = ["myorg", "core", "utils", "date"]
        assert self.strategy.is_match(p_no_ext, "myorg/core/utils/date", "/myorg/core/utils/date", parts) is True

    def test_scoped_package_lib_dir(self):
        """
        import { X } from '@myorg/lib-a'
        → libs/lib-a/src/index.ts
        """
        p_no_ext = "/project/libs/lib-a/src/index"
        parts = ["myorg", "lib-a"]
        assert self.strategy.is_match(p_no_ext, "myorg/lib-a", "/myorg/lib-a", parts) is True

    def test_scoped_package_apps_dir(self):
        """
        import { X } from '@myorg/web'
        → apps/web/src/index.ts
        """
        p_no_ext = "/project/apps/web/src/index"
        parts = ["myorg", "web"]
        assert self.strategy.is_match(p_no_ext, "myorg/web", "/myorg/web", parts) is True

    def test_unscoped_package_entry_point(self):
        """
        import { X } from 'shared-ui'
        → packages/shared-ui/src/index.ts
        """
        p_no_ext = "/project/packages/shared-ui/src/index"
        parts = ["shared-ui"]
        assert self.strategy.is_match(p_no_ext, "shared-ui", "/shared-ui", parts) is True

    def test_unscoped_package_subpath(self):
        """
        import { X } from 'shared-ui/button'
        → packages/shared-ui/src/button.ts
        """
        p_no_ext = "/project/packages/shared-ui/src/button"
        parts = ["shared-ui", "button"]
        assert self.strategy.is_match(p_no_ext, "shared-ui/button", "/shared-ui/button", parts) is True

    def test_unscoped_package_direct_index(self):
        """
        import { X } from 'shared-ui'
        → packages/shared-ui/index.ts  (без src/)
        """
        p_no_ext = "/project/packages/shared-ui/index"
        parts = ["shared-ui"]
        assert self.strategy.is_match(p_no_ext, "shared-ui", "/shared-ui", parts) is True

    def test_unscoped_package_lib_entry(self):
        """
        import { X } from 'shared-ui'
        → packages/shared-ui/lib/index.ts
        """
        p_no_ext = "/project/packages/shared-ui/lib/index"
        parts = ["shared-ui"]
        assert self.strategy.is_match(p_no_ext, "shared-ui", "/shared-ui", parts) is True

    def test_unscoped_package_subpath_no_src(self):
        """
        import { X } from 'utils/format'
        → packages/utils/format.ts  (напрямую, без src/)
        """
        p_no_ext = "/project/packages/utils/format"
        parts = ["utils", "format"]
        assert self.strategy.is_match(p_no_ext, "utils/format", "/utils/format", parts) is True

    def test_no_match_different_package(self):
        """Не должно совпадать, если пакет в import не соответствует пути."""
        p_no_ext = "/project/packages/other-ui/src/index"
        parts = ["myorg", "shared-ui"]
        assert self.strategy.is_match(p_no_ext, "myorg/shared-ui", "/myorg/shared-ui", parts) is False

    def test_no_match_non_monorepo_path(self):
        """Обычный src/ путь не должен срабатывать как монорепа."""
        p_no_ext = "/project/src/components/button"
        parts = ["components", "button"]
        assert self.strategy.is_match(p_no_ext, "components/button", "/components/button", parts) is False

    def test_no_match_external_lib(self):
        """node_modules не должна совпадать."""
        p_no_ext = "/project/node_modules/lodash/index"
        parts = ["lodash"]
        assert self.strategy.is_match(p_no_ext, "lodash", "/lodash", parts) is False

    def test_empty_parts(self):
        """Пустой список частей → False."""
        assert self.strategy.is_match("/foo/bar", "", "", []) is False

    def test_windows_path(self):
        """Windows-пути с \ должны обрабатываться корректно."""
        p_no_ext = "C:\\project\\packages\\shared-ui\\src\\index"
        parts = ["myorg", "shared-ui"]
        assert self.strategy.is_match(p_no_ext, "myorg/shared-ui", "/myorg/shared-ui", parts) is True

    def test_nested_scoped_package_subpath_with_scope(self):
        """
        import { X } from '@org/shared-ui/components/button'
        → packages/shared-ui/src/components/button.ts
        """
        p_no_ext = "/project/packages/shared-ui/src/components/button"
        parts = ["org", "shared-ui", "components", "button"]
        assert self.strategy.is_match(p_no_ext, "org/shared-ui/components/button", "/org/shared-ui/components/button", parts) is True

    def test_lerna_path(self):
        """
        import { X } from '@myco/logger'
        → packages/logger/src/index.ts  (Lerna layout)
        """
        p_no_ext = "/monorepo/packages/logger/src/index"
        parts = ["myco", "logger"]
        assert self.strategy.is_match(p_no_ext, "myco/logger", "/myco/logger", parts) is True

    def test_nx_path(self):
        """
        import { X } from '@myco/api'
        → libs/api/src/index.ts  (NX layout: libs/)
        """
        p_no_ext = "/monorepo/libs/api/src/index"
        parts = ["myco", "api"]
        assert self.strategy.is_match(p_no_ext, "myco/api", "/myco/api", parts) is True

    def test_turborepo_path(self):
        """
        import { X } from '@myco/web'
        → apps/web/src/index.ts  (Turborepo layout: apps/)
        """
        p_no_ext = "/monorepo/apps/web/src/index"
        parts = ["myco", "web"]
        assert self.strategy.is_match(p_no_ext, "myco/web", "/myco/web", parts) is True

    def test_modules_dir(self):
        """
        import { X } from 'my-module'
        → modules/my-module/index.ts  (modules/ layout)
        """
        p_no_ext = "/project/modules/my-module/index"
        parts = ["my-module"]
        assert self.strategy.is_match(p_no_ext, "my-module", "/my-module", parts) is True

    def test_resolve_pipeline(self):
        """
        Интеграционный тест: ImportResolutionService с MonorepoImportStrategy
        должен находить файлы монорепозитория.
        """
        svc = ImportResolutionService()

        available = [
            "/project/packages/shared-ui/src/index.ts",
            "/project/packages/shared-ui/src/button.tsx",
            "/project/packages/core/src/utils/format.ts",
            "/project/packages/core/src/index.ts",
            "/project/src/App.tsx",
            "/project/node_modules/react/index.js",
        ]

        matched = svc.resolve("@myorg/shared-ui", available)
        assert "/project/packages/shared-ui/src/index.ts" in matched

        matched2 = svc.resolve("@myorg/core/utils/format", available)
        assert "/project/packages/core/src/utils/format.ts" in matched2

    def test_resolve_no_false_positive(self):
        """Не должен находить node_modules или внешние пути."""
        svc = ImportResolutionService()

        available = [
            "/project/node_modules/lodash/index.js",
            "/project/node_modules/react/index.js",
            "/project/src/utils/date.ts",
        ]

        matched = svc.resolve("@myorg/shared-ui", available)
        assert matched == []

    def test_resolve_unscoped(self):
        """Импорт без @scope тоже должен работать."""
        svc = ImportResolutionService()

        available = [
            "/project/packages/shared-ui/src/index.ts",
            "/project/src/App.tsx",
        ]

        matched = svc.resolve("shared-ui", available)
        assert "/project/packages/shared-ui/src/index.ts" in matched
