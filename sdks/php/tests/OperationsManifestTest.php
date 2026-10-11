<?php

declare(strict_types=1);

namespace EdgeQuake\Tests;

use PHPUnit\Framework\TestCase;

final class OperationsManifestTest extends TestCase
{
    public function testEveryHttpOperationPathAndGeneratedVerb(): void
    {
        $root = dirname(__DIR__, 3);
        $ops = json_decode((string) file_get_contents($root . '/sdks/operation-manifest.json'), true);
        $generated = (string) file_get_contents($root . '/sdks/php/src/OperationsGenerated.php');
        $blob = '';
        foreach (glob($root . '/sdks/php/src/*.php') ?: [] as $file) {
            $blob .= (string) file_get_contents($file);
        }
        foreach ($ops as $op) {
            if ($op['kind'] === 'websocket') {
                $this->assertStringContainsString($op['path'], $generated, $op['path']);
                continue;
            }
            $this->assertStringContainsString($op['path'], $blob, $op['path']);
            if ($op['missing']['php']) {
                $this->assertStringContainsString($op['method'] . ' ' . $op['path'], $generated);
            }
        }
    }
}
